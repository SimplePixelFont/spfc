use spf::core::layout_from_data;
use spfc_abi::{BackendInfo, CURRENT_ABI_VERSION, CompileOptions, CompileResult, PluginOption};

mod builders;
use builders::*;
mod utilities;
use utilities::*;

#[spfc_abi::export]
fn get_backend_info() -> BackendInfo {
    BackendInfo {
        name: "P8 PICO8 Backend",
        version: 1,
        abi_version: CURRENT_ABI_VERSION,
    }
}

#[spfc_abi::export]
fn get_plugin_options() -> Vec<PluginOption> {
    vec![]
}

#[spfc_abi::export]
fn compile(options: CompileOptions) -> CompileResult {
    let data = std::fs::read(&options.input).unwrap();
    let layout = layout_from_data(&data).unwrap();
    let font_table = layout.font_tables.first().unwrap();
    let font = font_table.fonts.first().unwrap();

    let mut process = Process::default();
    process.family_name = font.name.clone();
    process.family_version = font.version as f64;
    process.manufacturer = font.author.clone();
    process.pixmap_pairs = create_pixmap_pairs(&layout);

    let font_data = create_program_string(&process).unwrap();
    std::fs::write(&options.output, &font_data).unwrap();

    println!(
        "Finished writing {} bytes to {}",
        font_data.len(),
        options.output
    );

    CompileResult::Success
}