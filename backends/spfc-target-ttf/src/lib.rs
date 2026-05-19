use spf::core::layout_from_data;
use spfc_abi::{BackendInfo, CURRENT_ABI_VERSION, CompileOptions, CompileResult, PluginOption};
use anyhow::{Result, anyhow, ensure};

mod builders;
use builders::*;
mod utilities;
use utilities::*;

// Source - https://stackoverflow.com/a/62759540
// Posted by vallentin, modified by community. See post 'Timeline' for change history
// Retrieved 2026-05-19, License - CC BY-SA 4.0
#[non_exhaustive]
struct ExtraArguments;

impl ExtraArguments {
    pub const COPYRIGHT: &'static str = "copyright";
    pub const VENDOR_URL: &'static str = "vendor-url";
    pub const LICENSE_DESCRIPTION: &'static str = "license-description";
    pub const PIXEL_SIZE: &'static str = "pixel-size";
    pub const DESCENDER_PIXELS: &'static str = "descender-pixels";
}


#[spfc_abi::export]
fn get_backend_info() -> BackendInfo {
    BackendInfo {
        name: "TTF TrueType Backend",
        version: 2,
        abi_version: CURRENT_ABI_VERSION,
    }
}

#[spfc_abi::export]
fn get_plugin_options() -> Vec<PluginOption> {
    vec![
        PluginOption {
            name: ExtraArguments::COPYRIGHT,
            description: "Set the font's copyright metadata",
            default_value: "Copyright (c) 2026 SimplePixelFont",
        },
        PluginOption {
            name: ExtraArguments::VENDOR_URL,
            description: "Set the font's vendor URL metadata",
            default_value: "https://github.com/SimplePixelFont",
        },
        PluginOption {
            name: ExtraArguments::LICENSE_DESCRIPTION,
            description: "Set the font's license description metadata",
            default_value: "Licensed under the Apache License, Version 2.0",
        },
        PluginOption {
            name: ExtraArguments::PIXEL_SIZE,
            description: "Pixel size in font units",
            default_value: "64",
        },
        PluginOption {
            name: ExtraArguments::DESCENDER_PIXELS,
            description: "Descender size in pixels",
            default_value: "0",
        },
    ]
}

#[spfc_abi::export]
fn compile(options: CompileOptions) -> Result<CompileResult> {
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

    if let Some(copyright) = options
        .get_extra_argument(ExtraArguments::COPYRIGHT) {
            process.copyright = copyright.to_string();
        }
    if let Some(vendor_url) = options
        .get_extra_argument(ExtraArguments::VENDOR_URL) {
            process.vendor_url = vendor_url.to_string();
        }
    if let Some(license_description) = options
        .get_extra_argument(ExtraArguments::LICENSE_DESCRIPTION) {
            process.license_description = license_description.to_string();
        }
    if let Some(target_pixel_size) = options
        .get_extra_argument(ExtraArguments::PIXEL_SIZE) {
            let parsed = target_pixel_size.parse::<i16>()?;
            ensure!(parsed > 0, "`pixel-size` must be greater than 0");
            process.target_pixel_size = parsed;
        }
    if let Some(descender_pixels) = options
        .get_extra_argument(ExtraArguments::DESCENDER_PIXELS) {
            let parsed = descender_pixels.parse::<i16>()?;
            ensure!(parsed >= 0, "`descender-pixels` must be non-negative");
            process.descender_pixels = parsed;
        }
    ensure!(
        process.descender_pixels < process.max_pixel_height,
        "`descender-pixels` must be smaller than the glyph height"
    );

    process.pixmap_pairs = create_pixmap_pairs(&layout);
    process.max_pixel_width = max_width(&process.pixmap_pairs);
    process.max_pixel_height = max_height(&process.pixmap_pairs);
    process.units_per_em = calculate_units_per_em(
        process.max_pixel_width,
        process.max_pixel_height,
        process.target_pixel_size,
    );

    process.add_required_whitespace(); // a separate validation layer might be needed later. Although really the only character that needs fixing and only if space exists :)
    process.ensure_ligature_components();
    process.prepare_color_font_data(&layout);
    process.update_max_points_and_contours()?;
    process.is_monospaced = is_monospaced(&process.pixmap_pairs);

    push_head_table(&mut process)?;
    push_hhea_table(&mut process)?;
    push_maxp_table(&mut process)?;
    push_os2_table(&mut process)?;
    push_name_table(&mut process)?;
    push_post_table(&mut process)?;
    push_glyf_loca_tables(&mut process)?;
    push_hmtx_table(&mut process)?;
    push_cmap_table(&mut process)?;
    push_gasp_table(&mut process)?;
    push_gsub_table(&mut process)?;
    push_colr_cpal_tables(&mut process)?;

    process.builder.add_raw(
        write_fonts::types::Tag::new(b"prep"),
        vec![0xB8, 0x01, 0xFF, 0x85, 0xB0, 0x04, 0x8D],
    );

    // store raw spf data in a custom table for potential future use.
    // This is not necessary for the font to function, but it allows the original data to be preserved inside the font file itself,
    // which can be useful to debug.
    process.builder.add_raw(
        write_fonts::types::Tag::new(b"bspf"),
        spf::core::layout_to_data(&layout).expect("Could not add bspf table due to serialization error"),
    );

    let font_data = process.builder.build();
    std::fs::write(&options.output, &font_data)?;

    println!(
        "Finished writing {} bytes to {}",
        font_data.len(),
        options.output
    );
    Ok(CompileResult::Success)
}
