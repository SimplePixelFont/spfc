use std::ffi::{CStr, CString};

use crate::{
    ABIBackendInfo, ABICompileOptions, ABIKeyValuePair, ABIPluginOption, ABIPluginOptionsList, BackendInfo, CompileOptions, CompileResult, KeyValuePair, PluginOption, PluginOptionsList, convertors::ConversionError
};

/// Links an ergonomic Rust type to its `#[repr(C)]` counterpart.
///
/// Implement this on your Rust-side structs:
///
/// ```ignore
/// impl CRepr for MyInput {
///     type C = CMyInput;
/// }
/// ```
pub trait ABIRepr {
    type ABI: Sized;
}

impl<S: ToString> ABIRepr for BackendInfo<S> {
    type ABI = ABIBackendInfo;
}

impl<S: ToString> ABIRepr for PluginOption<S> {
    type ABI = ABIPluginOption;
}

impl<S: ToString> ABIRepr for Vec<PluginOption<S>> {
    type ABI = ABIPluginOptionsList;
}

impl<S: ToString> ABIRepr for PluginOptionsList<S> {
    type ABI = ABIPluginOptionsList;
}

impl ABIRepr for KeyValuePair {
    type ABI = ABIKeyValuePair;
}

impl ABIRepr for CompileOptions {
    type ABI = ABICompileOptions;
}

impl ABIRepr for CompileResult {
    type ABI = u8;
}

// ---------------------------------------------------------------------------
// C-safe result envelope
// ---------------------------------------------------------------------------

/// Returned by every generated `extern "C"` plugin function.
///
/// - On success: `data` is a heap-allocated `Box<T>` cast to `*mut c_void`,
///   `error` is null.
/// - On failure: `data` is null, `error` is a heap-allocated C string.
///
/// The *host* is responsible for freeing both pointers via the provided
/// `plugin_free_result` function.
#[repr(C)]
pub struct ABIResult {
    pub data: *mut std::ffi::c_void,
    pub error: *mut std::ffi::c_char,
}

impl ABIResult {
    /// Construct a successful result, boxing `val` on the heap.
    pub fn ok<T>(val: T) -> Self {
        let boxed = Box::new(val);
        ABIResult {
            data: Box::into_raw(boxed) as *mut std::ffi::c_void,
            error: std::ptr::null_mut(),
        }
    }

    /// Construct an error result from anything that converts to a string.
    pub fn err<E: ToString>(e: E) -> Self {
        let msg = CString::new(e.to_string())
            .unwrap_or_else(|_| CString::new("(error message contained null byte)").unwrap());
        ABIResult {
            data: std::ptr::null_mut(),
            error: msg.into_raw(),
        }
    }

    /// Convenience: did the call succeed?
    pub fn is_ok(&self) -> bool {
        self.error.is_null()
    }
}

#[derive(Debug)]
pub enum UnpackResultError {
    ConversionError(ConversionError),
    ErrorMessage(String),
}

// Implement `Display` for user-friendly messages
impl std::fmt::Display for UnpackResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnpackResultError::ConversionError(e) => write!(f, "ConversionError: {}", e),
            UnpackResultError::ErrorMessage(msg) => write!(f, "ErrorMessage: {}", msg),
        }
    }
}

impl std::error::Error for UnpackResultError {}

pub unsafe fn unpack_result<T>(raw: ABIResult) -> Result<T, UnpackResultError>
where
    T: ABIRepr,
    T::ABI: TryInto<T>, // C -> Rust conversion using TryInto
    <T::ABI as TryInto<T>>::Error: Into<ConversionError>,
{
    unsafe {
        if !raw.error.is_null() {
            let msg = CStr::from_ptr(raw.error).to_string_lossy().into_owned();
            //plugin_free_result(raw);
            return Err(UnpackResultError::ErrorMessage(msg));
        }

        let c_val = *Box::from_raw(raw.data as *mut T::ABI);
        //plugin_free_result(raw); // error ptr is null, just zeroes out
        let result = c_val.try_into().map_err(|e| UnpackResultError::ConversionError(e.into()))?;
        
        Ok(result)
    }
}

/// Free the pointers inside a [`CResult`].
///
/// The host **must** call this after consuming the result.
///
/// # Safety
/// Both pointers must have originated from a `CResult` produced by this
/// library, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_free_result(result: ABIResult) {
    unsafe {
        if !result.data.is_null() {
            // We don't know the concrete type here, so the host is expected to
            // have already read / moved out of `data` before freeing.
            // Casting to `u8` just drops the allocation.
            drop(Box::from_raw(result.data as *mut u8));
        }
        if !result.error.is_null() {
            drop(CString::from_raw(result.error));
        }
    }
}
