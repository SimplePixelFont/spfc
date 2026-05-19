use anyhow::{Error, anyhow};
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
fn compile(options: CompileOptions) -> Result<CompileResult, Error> {
    let data = std::fs::read(&options.input)?;
    let layout = layout_from_data(&data)
        .map_err(|e| anyhow!("Failed to parse input data into layout: {e:?}"))?;
    let font_table = layout
        .font_tables
        .first()
        .ok_or_else(|| anyhow!("No font tables found"))?;
    let font = font_table
        .fonts
        .first()
        .ok_or_else(|| anyhow!("No fonts found in font table"))?;

    let mut process = Process::default();
    process.family_name = font.name.clone();
    process.family_version = font.version as f64;
    process.manufacturer = font.author.clone();
    process.pixmap_pairs = create_pixmap_pairs(&layout);

    let font_data = create_program_string(&process)?;
    std::fs::write(&options.output, &font_data)?;

    println!(
        "Finished writing {} bytes to {}",
        font_data.len(),
        options.output
    );

    Ok(CompileResult::Success)
}