use core::ffi::*;
use core::str::Utf8Error;
use std::ffi::CString;
use std::ffi::NulError;

use crate::ABIBackendInfo;
use crate::ABICompileOptions;
use crate::ABIKeyValuePair;
use crate::ABIPluginOption;
use crate::ABIPluginOptionsList;
use crate::CompileOptions;
use crate::CompileResult;
use crate::KeyValuePair;
use crate::PluginOption;
use crate::PluginOptionsList;
use crate::rust_types::BackendInfo;

// Macro to convert a Vec into a raw pointer and length
#[macro_export]
macro_rules! vec_to_raw {
    ($vec:expr) => {{
        let len = $vec.len();
        let ptr = if len == 0 {
            core::ptr::null_mut()
        } else {
            let mut boxed = $vec.into_boxed_slice();
            let ptr = boxed.as_mut_ptr();
            core::mem::forget(boxed);
            ptr
        };
        (ptr, len)
    }};
}

#[macro_export]
// Macro to convert a Vec with element conversion into a raw pointer and length.
// Used for vectors with elements of structs.
macro_rules! vec_to_raw_with_conversion {
    ($vec:expr, $item_type:ty) => {{
        let len = $vec.len();
        let mut converted: Vec<$item_type> = Vec::with_capacity(len);
        for item in $vec {
            converted.push(item.try_into()?);
        }
        vec_to_raw!(converted)
    }};
}

#[macro_export]
// Macro to reconstruct a Vec from raw pointer and length, given the vector has struct elements.
macro_rules! vec_from_raw_with_conversion {
    ($ptr:expr, $len:expr) => {{
        let len = $len as usize;
        let mut vec = Vec::with_capacity(len);
        for index in 0..len {
            let item = &*$ptr.add(index);
            vec.push(item.try_into()?);
        }
        vec
    }};
}

#[derive(Debug, Clone)]
pub enum ConversionError {
    NulError(NulError),
    Utf8Error(Utf8Error),
    UnsupportedCompileResultValue(u8),
}
impl std::error::Error for ConversionError {}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::NulError(e) => write!(f, "NulError: {}", e),
            ConversionError::Utf8Error(e) => write!(f, "Utf8Error: {}", e),
            ConversionError::UnsupportedCompileResultValue(e) => write!(f, "UnsupportedCompileResultValue: {}", e),
        }
    }
}

impl From<NulError> for ConversionError {
    fn from(err: NulError) -> Self {
        ConversionError::NulError(err)
    }
}

impl From<Utf8Error> for ConversionError {
    fn from(err: Utf8Error) -> Self {
        ConversionError::Utf8Error(err)
    }
}

impl<S: ToString> TryFrom<BackendInfo<S>> for ABIBackendInfo {
    type Error = ConversionError;

    fn try_from(value: BackendInfo<S>) -> Result<Self, Self::Error> {
        let name = CString::new(value.name.to_string().as_str())?;
        let name_ptr = name.into_raw() as *const c_char;

        Ok(ABIBackendInfo {
            name: name_ptr,
            version: value.version,
            abi_version: value.abi_version,
        })
    }
}

impl TryInto<BackendInfo<String>> for ABIBackendInfo {
    type Error = ConversionError;

    fn try_into(self) -> Result<BackendInfo<String>, Self::Error> {
        unsafe {
            let name = CStr::from_ptr(self.name).to_str()?.to_owned();

            Ok(BackendInfo {
                name,
                version: self.version,
                abi_version: self.abi_version,
            })
        }
    }
}

impl<S: ToString> TryFrom<PluginOption<S>> for ABIPluginOption {
    type Error = ConversionError;

    fn try_from(value: PluginOption<S>) -> Result<Self, Self::Error> {
        let name = CString::new(value.name.to_string().as_str())?;
        let name_ptr = name.into_raw() as *const c_char;

        let description = CString::new(value.description.to_string().as_str())?;
        let description_ptr = description.into_raw() as *const c_char;

        let default_value = CString::new(value.default_value.to_string().as_str())?;
        let default_value_ptr = default_value.into_raw() as *const c_char;

        Ok(ABIPluginOption {
            name: name_ptr,
            description: description_ptr,
            default_value: default_value_ptr,
        })
    }
}

impl TryInto<PluginOption<String>> for &ABIPluginOption {
    type Error = ConversionError;

    fn try_into(self) -> Result<PluginOption<String>, Self::Error> {
        unsafe {
            let name = CStr::from_ptr(self.name).to_str()?.to_owned();

            let description = CStr::from_ptr(self.description).to_str()?.to_owned();

            let default_value = CStr::from_ptr(self.default_value).to_str()?.to_owned();

            Ok(PluginOption {
                name,
                description,
                default_value,
            })
        }
    }
}

impl<S: ToString> TryFrom<PluginOptionsList<S>> for ABIPluginOptionsList {
    type Error = ConversionError;

    fn try_from(value: PluginOptionsList<S>) -> Result<Self, Self::Error> {
        let (options_ptr, length) = vec_to_raw_with_conversion!(value.options, ABIPluginOption);
        Ok(ABIPluginOptionsList {
            options: options_ptr,
            options_length: length as c_ulong,
        })
    }
}

impl TryInto<PluginOptionsList<String>> for ABIPluginOptionsList {
    type Error = ConversionError;

    fn try_into(self) -> Result<PluginOptionsList<String>, Self::Error> {
        unsafe {
            let options: Vec<PluginOption<String>> = vec_from_raw_with_conversion!(self.options, self.options_length);
            Ok(PluginOptionsList { options: options })
        }
    }
}




impl<S: ToString> TryFrom<Vec<PluginOption<S>>> for ABIPluginOptionsList {
    type Error = ConversionError;

    fn try_from(value: Vec<PluginOption<S>>) -> Result<Self, Self::Error> {
        let (options_ptr, length) = vec_to_raw_with_conversion!(value, ABIPluginOption);
        Ok(ABIPluginOptionsList {
            options: options_ptr,
            options_length: length as c_ulong,
        })
    }
}

impl TryInto<Vec<PluginOption<String>>> for ABIPluginOptionsList {
    type Error = ConversionError;

    fn try_into(self) -> Result<Vec<PluginOption<String>>, Self::Error> {
        unsafe {
            Ok(vec_from_raw_with_conversion!(self.options, self.options_length))
        }
    }
}





impl TryFrom<KeyValuePair> for ABIKeyValuePair {
    type Error = ConversionError;
    
    fn try_from(value: KeyValuePair) -> Result<Self, Self::Error> {
        let key = CString::new(value.key.as_str())?;
        let key_ptr = key.into_raw() as *const c_char;

        let value = CString::new(value.value.as_str())?;
        let value_ptr = value.into_raw() as *const c_char;

        Ok(ABIKeyValuePair {
            key: key_ptr,
            value: value_ptr,
        })
    }
}

impl TryInto<KeyValuePair> for &ABIKeyValuePair {
    type Error = ConversionError;

    fn try_into(self) -> Result<KeyValuePair, Self::Error> {
        unsafe {
            let key = CStr::from_ptr(self.key).to_str()?.to_owned();
            let value = CStr::from_ptr(self.value).to_str()?.to_owned();

            Ok(KeyValuePair { key, value })
        }
    }
}

impl TryFrom<CompileOptions> for ABICompileOptions {
    type Error = ConversionError;

    fn try_from(value: CompileOptions) -> Result<Self, Self::Error> {
        let input = CString::new(value.input.as_str())?;
        let input_ptr = input.into_raw() as *const c_char;

        let output = CString::new(value.output.as_str())?;
        let output_ptr = output.into_raw() as *const c_char;

        let (extra_args_ptr, extra_args_length) = vec_to_raw_with_conversion!(value.extra_arguments, ABIKeyValuePair);

        Ok(ABICompileOptions {
            input: input_ptr,
            output: output_ptr,
            extra_args: extra_args_ptr,
            extra_args_length: extra_args_length as c_ulong,
        })
    }
}

impl TryInto<CompileOptions> for &ABICompileOptions {
    type Error = ConversionError;

    fn try_into(self) -> Result<CompileOptions, Self::Error> {
        unsafe {
            let input = CStr::from_ptr(self.input).to_str()?.to_owned();
            let output = CStr::from_ptr(self.output).to_str()?.to_owned();

            let extra_args = vec_from_raw_with_conversion!(self.extra_args, self.extra_args_length);

            Ok(CompileOptions {
                input,
                output,
                extra_arguments: extra_args,
            })
        }
    }
}

impl TryFrom<CompileResult> for u8 {
    type Error = ConversionError;

    fn try_from(value: CompileResult) -> Result<Self, Self::Error> {
        match value {
            CompileResult::Success => Ok(0),
            CompileResult::Failure => Ok(1),
        }
    }
}

impl TryInto<CompileResult> for u8 {
    type Error = ConversionError;

    fn try_into(self) -> Result<CompileResult, Self::Error> {
        match self {
            0 => Ok(CompileResult::Success),
            1 => Ok(CompileResult::Failure),
            _ => Err(ConversionError::UnsupportedCompileResultValue(self)),
        }
    }
}