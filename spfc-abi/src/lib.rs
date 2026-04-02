use core::ffi::*;
pub const CURRENT_ABI_VERSION: u32 = 1;

pub mod convertors;
pub mod rust_types;
pub mod result;
pub use spfc_macros::export;

pub use rust_types::*;
pub use result::*;

#[repr(C)]
pub struct ABIBackendInfo {
    pub name: *const c_char,
    pub version: u32,
    pub abi_version: u32,
}

/// Defines a custom CLI flag that the plugin supports
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ABIPluginOption {
    pub name: *const c_char,          // e.g., "hinting"
    pub description: *const c_char,   // e.g., "Set TTF hinting level"
    pub default_value: *const c_char, // e.g., "none" (empty if no default)
}

/// A parsed key-value pair passed back to the plugin
#[repr(C)]
pub struct ABIKeyValuePair {
    pub key: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct ABICompileOptions {
    pub input: *const c_char,
    pub output: *const c_char,
    pub extra_args: *const ABIKeyValuePair,
    pub extra_args_length: c_ulong,
}

#[repr(C)]
pub struct ABIPluginOptionsList {
    pub options: *const ABIPluginOption,
    pub options_length: c_ulong,
}

// Function signatures
pub type GetBackendInfoFn = extern "C" fn() -> ABIResult;
pub type GetPluginOptionsFn = extern "C" fn() -> ABIResult;
pub type CompileFn = extern "C" fn(
    options: ABICompileOptions,
) -> ABIResult;