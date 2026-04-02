use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    FnArg, GenericArgument, Ident, ItemFn, Pat, PathArguments, ReturnType, Type, parse_macro_input,
};

// ---------------------------------------------------------------------------
// #[plugin_fn]
// ---------------------------------------------------------------------------
//
// Transforms:
//
//   #[plugin_fn]
//   pub fn process(input: MyInput, cfg: MyConfig) -> Result<MyOutput, MyError> {
//       // ergonomic Rust body
//   }
//
// Into:
//
//   // The real FFI export — receives/returns C types, handles all conversion
//   // and panic safety.
//   #[no_mangle]
//   pub extern "C" fn process(
//       input: <MyInput as ::plugin_api::CRepr>::C,
//       cfg:   <MyConfig as ::plugin_api::CRepr>::C,
//   ) -> ::plugin_api::CResult {
//       // catch_unwind so panics never unwind across the FFI boundary
//       let __result = ::std::panic::catch_unwind(|| {
//           let input = match MyInput::try_from(input) { Ok(v) => v, Err(e) => return ... };
//           let cfg   = match MyConfig::try_from(cfg)   { Ok(v) => v, Err(e) => return ... };
//           match _inner_process(input, cfg) {
//               Ok(out) => match ::std::convert::TryInto::try_into(out) {
//                   Ok(c)  => ::plugin_api::CResult::ok(c),
//                   Err(e) => ::plugin_api::CResult::err(e.to_string()),
//               },
//               Err(e) => ::plugin_api::CResult::err(e.to_string()),
//           }
//       });
//       match __result {
//           Ok(r)  => r,
//           Err(_) => ::plugin_api::CResult::err("plugin panicked"),
//       }
//   }
//
//   // The original function, renamed so the export can call it.
//   fn _inner_process(input: MyInput, cfg: MyConfig) -> Result<MyOutput, MyError> {
//       // original body
//   }

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // -----------------------------------------------------------------------
    // 1. Extract pieces of the original signature
    // -----------------------------------------------------------------------
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let body = &input_fn.block;
    let inner_name = Ident::new(&format!("_inner_{}", fn_name), Span::call_site());

    // -----------------------------------------------------------------------
    // 2. Parse arguments — we need (pattern, type) pairs.
    //    We reject `self` receivers; plugins are free functions.
    // -----------------------------------------------------------------------
    let args: Vec<(&Pat, &Type)> = input_fn
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(pt) => (pt.pat.as_ref(), pt.ty.as_ref()),
            FnArg::Receiver(_) => {
                panic!("#[export] cannot be applied to methods — use a free function")
            }
        })
        .collect();

    // -----------------------------------------------------------------------
    // 3. Build the outer FFI parameter list:
    //    Each `name: RustType` becomes `name: <RustType as ::plugin_api::CRepr>::C`
    // -----------------------------------------------------------------------
    let ffi_params = args.iter().map(|(pat, ty)| {
        quote! {
            #pat : <#ty as ::spfc_abi::ABIRepr>::ABI
        }
    });

    // -----------------------------------------------------------------------
    // 4. Build the TryFrom conversions at the top of the outer function body.
    //    For each argument we emit:
    //
    //      let name = match <RustType>::try_from(name) {
    //          Ok(v)  => v,
    //          Err(e) => return ::plugin_api::CResult::err(e.to_string()),
    //      };
    // -----------------------------------------------------------------------
    let arg_conversions = args.iter().map(|(pat, ty)| {
        // quote! {
        //     let #pat = match <#ty as ::std::convert::TryFrom<<#ty as ::spfc_abi::ABIRepr>::ABI>>::try_from(#pat) {
        //         Ok(__v)  => __v,
        //         Err(__e) => return ::spfc_abi::ABIResult::err(__e.to_string()),
        //     };
        // }
        quote! {
            let #pat = match ::std::convert::TryInto::<#ty>::try_into(&#pat) {
                Ok(__v)  => __v,
                Err(__e) => return ::spfc_abi::ABIResult::err(__e.to_string()),
            };
        }
    });

    // -----------------------------------------------------------------------
    // 5. Re-assemble the argument names for the call to the inner function.
    // -----------------------------------------------------------------------
    let arg_names = args.iter().map(|(pat, _)| pat);

    // -----------------------------------------------------------------------
    // 6. Parse the return type.
    //
    //    We support two forms:
    //      a) `-> Result<OkType, ErrType>`   (most common)
    //      b) `-> SomeType`                  (infallible — wrapped in Result internally)
    //
    //    In both cases the outer function returns `::plugin_api::CResult`.
    // -----------------------------------------------------------------------
    let return_type = &input_fn.sig.output;

    // Try to detect `Result<O, E>` and extract `O`.
    let result_arm = match return_type {
        ReturnType::Default => {
            // fn foo() — inner returns (), nothing to convert
            quote! {
                #inner_name(#(#arg_names),*);
                ::spfc_abi::ABIResult::ok(())
            }
        }
        ReturnType::Type(_, ty) => {
            if let Some(ok_type) = extract_result_ok_type(ty) {
                // -> Result<OkType, ErrType>
                quote! {
                    match #inner_name(#(#arg_names),*) {
                        Ok(__out) => {
                            match <
                                <#ok_type as ::spfc_abi::ABIRepr>::ABI
                                as ::std::convert::TryFrom<#ok_type>
                            >::try_from(__out) {
                                Ok(__c)  => ::spfc_abi::ABIResult::ok(__c),
                                Err(__e) => ::spfc_abi::ABIResult::err(__e.to_string()),
                            }
                        },
                        Err(__e) => ::spfc_abi::ABIResult::err(__e.to_string()),
                    }
                }
            } else {
                // -> SomeType  (infallible)
                quote! {
                    let __out = #inner_name(#(#arg_names),*);
                    match <
                        <#ty as ::spfc_abi::ABIRepr>::ABI
                        as ::std::convert::TryFrom<#ty>
                    >::try_from(__out) {
                        Ok(__c)  => ::spfc_abi::ABIResult::ok(__c),
                        Err(__e) => ::spfc_abi::ABIResult::err(__e.to_string()),
                    }
                }
            }
        }
    };

    // -----------------------------------------------------------------------
    // 7. Reconstruct the original inputs for the inner function signature.
    //    We keep the original types and patterns unchanged.
    // -----------------------------------------------------------------------
    let original_inputs = &input_fn.sig.inputs;

    // -----------------------------------------------------------------------
    // 8. Emit the final token stream:
    //      - the `extern "C"` export (with catch_unwind)
    //      - the original function renamed to _inner_<name>
    // -----------------------------------------------------------------------
    let expanded = quote! {
        // ── FFI export ──────────────────────────────────────────────────────
        #[unsafe(no_mangle)]
        #vis extern "C" fn #fn_name(#(#ffi_params),*) -> ::spfc_abi::ABIResult {
            // Catch any Rust panics — they must never unwind across the FFI
            // boundary as that is undefined behaviour.
            let __outcome = ::std::panic::catch_unwind(
                ::std::panic::AssertUnwindSafe(move || {
                    // Convert every C argument to its ergonomic Rust counterpart.
                    #(#arg_conversions)*

                    // Call the real implementation and convert the return value.
                    #result_arm
                })
            );

            match __outcome {
                Ok(__r)  => __r,
                Err(__p) => {
                    // Try to extract a useful panic message.
                    let __msg = if let Some(__s) = __p.downcast_ref::<&str>() {
                        format!("plugin panicked: {}", __s)
                    } else if let Some(__s) = __p.downcast_ref::<String>() {
                        format!("plugin panicked: {}", __s)
                    } else {
                        "plugin panicked (unknown cause)".to_string()
                    };
                    ::spfc_abi::ABIResult::err(__msg)
                }
            }
        }

        // ── Inner implementation (ergonomic Rust) ───────────────────────────
        fn #inner_name(#original_inputs) #return_type {
            #body
        }
    };

    expanded.into()
}

// ---------------------------------------------------------------------------
// Helper: given a `Type`, check whether it is `Result<O, E>` and if so
// return the `O` type. Returns `None` for everything else.
// ---------------------------------------------------------------------------
fn extract_result_ok_type(ty: &Type) -> Option<&Type> {
    // We're looking for a path type whose last segment is "Result" with
    // two angle-bracket generic arguments.
    let Type::Path(type_path) = ty else {
        return None;
    };

    let last = type_path.path.segments.last()?;
    if last.ident != "Result" {
        return None;
    }

    let PathArguments::AngleBracketed(ref args) = last.arguments else {
        return None;
    };

    // First generic argument should be the `Ok` type.
    if let Some(GenericArgument::Type(ok_ty)) = args.args.first() {
        Some(ok_ty)
    } else {
        None
    }
}
