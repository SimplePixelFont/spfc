

pub struct BackendInfo<S: ToString = &'static str> {
    pub name: S,
    pub version: u32,
    pub abi_version: u32,
}

/// Defines a custom CLI flag that the plugin supports
#[derive(Debug, Clone)]
pub struct PluginOption<S: ToString = &'static str> {
    pub name: S,
    pub description: S,
    pub default_value: S,
}

pub struct PluginOptionsList<S: ToString = &'static str> {
    pub options: Vec<PluginOption<S>>,
}

/// A parsed key-value pair passed back to the plugin
#[derive(Debug, Clone)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct CompileOptions {
    pub input: String,
    pub output: String,

    pub extra_arguments: Vec<KeyValuePair>,
}

impl CompileOptions {
    pub fn get_extra_argument(&self, key: &str) -> Option<&str> {
        self.extra_arguments.iter()
            .find(|kv| kv.key == key)
            .map(|kv| kv.value.as_str())
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum CompileResult {
    Success,
    Failure
}