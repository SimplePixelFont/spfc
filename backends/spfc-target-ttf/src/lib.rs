use spf::core::layout_from_data;
use spfc_abi::{BackendInfo, CURRENT_ABI_VERSION, CompileOptions, CompileResult, PluginOption};

mod builders;
use builders::*;
mod utilities;
use utilities::*;

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
            name: "copyright",
            description: "Set the font's copyright metadata",
            default_value: "Copyright (c) 2026 SimplePixelFont",
        },
        PluginOption {
            name: "vendor-url",
            description: "Set the font's vendor URL metadata",
            default_value: "https://github.com/SimplePixelFont",
        },
        PluginOption {
            name: "license-description",
            description: "Set the font's license description metadata",
            default_value: "Licensed under the Apache License, Version 2.0",
        },
        PluginOption {
            name: "pixel-size",
            description: "Pixel size in font units",
            default_value: "64",
        },
        PluginOption {
            name: "descender-pixels",
            description: "Descender size in pixels",
            default_value: "0",
        },
    ]
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
    process.copyright = options.get_extra_argument("copyright").unwrap().to_owned();
    process.manufacturer = font.author.clone();
    process.vendor_url = options.get_extra_argument("vendor-url").unwrap().to_owned();
    process.license_description = options
        .get_extra_argument("license-description")
        .unwrap()
        .to_owned();

    process.target_pixel_size = options
        .get_extra_argument("pixel-size")
        .unwrap()
        .parse::<i16>()
        .unwrap();
    process.descender_pixels = options
        .get_extra_argument("descender-pixels")
        .unwrap()
        .parse::<i16>()
        .unwrap();

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
    process.update_max_points_and_contours();
    process.is_monospaced = is_monospaced(&process.pixmap_pairs);

    push_head_table(&mut process).unwrap();
    push_hhea_table(&mut process).unwrap();
    push_maxp_table(&mut process).unwrap();
    push_os2_table(&mut process).unwrap();
    push_name_table(&mut process).unwrap();
    push_post_table(&mut process).unwrap();
    push_glyf_loca_tables(&mut process).unwrap();
    push_hmtx_table(&mut process).unwrap();
    push_cmap_table(&mut process).unwrap();
    push_gasp_table(&mut process).unwrap();
    push_gsub_table(&mut process).unwrap();
    push_colr_cpal_tables(&mut process).unwrap();

    process.builder.add_raw(
        write_fonts::types::Tag::new(b"prep"),
        vec![0xB8, 0x01, 0xFF, 0x85, 0xB0, 0x04, 0x8D],
    );

    // store raw spf data in a custom table for potential future use.
    // This is not necessary for the font to function, but it allows the original data to be preserved inside the font file itself,
    // which can be useful to debug.
    process.builder.add_raw(
        write_fonts::types::Tag::new(b"bspf"),
        spf::core::layout_to_data(&layout).unwrap(),
    );

    let font_data = process.builder.build();
    std::fs::write(&options.output, &font_data).unwrap();

    println!(
        "Finished writing {} bytes to {}",
        font_data.len(),
        options.output
    );
    CompileResult::Success
}
