// use spf::core::{Character, CharacterTable, Color, ColorTable, Font, FontTable, FontType, Layout, Pixmap, PixmapTable};
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
            description: "Decender size in pixels",
            default_value: "0",
        },
    ]
}

// fn sample_pixmap_table() -> PixmapTable {
//     PixmapTable {
//         constant_width: None,
//         constant_height: Some(4),
//         constant_bits_per_pixel: Some(2),
//         color_table_indexes: Some(vec![0]),
//         pixmaps: vec![
//             Pixmap {
//                 custom_width: Some(4),
//                 custom_height: None,
//                 custom_bits_per_pixel: None,
//                 data: vec![0b00010100, 0b01000001, 0b01000001, 0b00010100],
//             },
//             Pixmap {
//                 custom_width: Some(5),
//                 custom_height: None,
//                 custom_bits_per_pixel: None,
//                 data: vec![0b00010001, 0b01000101, 0b00010100, 0b01010001, 0b01010101],
//             },
//             Pixmap {
//                 custom_width: Some(4),
//                 custom_height: None,
//                 custom_bits_per_pixel: None,
//                 data: vec![0b00011000, 0b00011000, 0b01000001, 0b00010100],
//             },
//             // Pixmap {
//             //     custom_width: Some(4),
//             //     custom_height: None,
//             //     custom_bits_per_pixel: None,
//             //     data: vec![0b11110001, 0b10001111],
//             // },
//         ],
//     }
// }

// fn sample_color_table() -> ColorTable {
//     ColorTable {
//         use_color_type: false,
//         constant_alpha: None,
//         colors: vec![
//             Color {
//                 color_type: None,
//                 custom_alpha: Some(0),
//                 r: 255,
//                 g: 255,
//                 b: 255,
//             },
//             Color {
//                 color_type: None,
//                 custom_alpha: Some(255),
//                 r: 0,
//                 g: 0,
//                 b: 0,
//             },
//             Color {
//                 color_type: None,
//                 custom_alpha: Some(255),
//                 r: 0,
//                 g: 255,
//                 b: 0,
//             },
//         ],
//     }
// }

// fn sample_layout() -> Layout {
//         let mut font = Layout::default();

//         font.character_tables = vec![CharacterTable {
//             use_advance_x: false,
//             use_pixmap_index: false,
//             use_pixmap_table_index: false,
//             constant_cluster_codepoints: None,
//             pixmap_table_indexes: Some(vec![0]),
//             characters: vec![
//                 Character {
//                     advance_x: None,
//                     pixmap_index: None,
//                     pixmap_table_index: None,
//                     grapheme_cluster: "o".to_string(),
//                 },
//                 Character {
//                     advance_x: None,
//                     pixmap_index: None,
//                     pixmap_table_index: None,
//                     grapheme_cluster: "w".to_string(),
//                 },
//                 Character {
//                     advance_x: None,
//                     pixmap_index: None,
//                     pixmap_table_index: None,
//                     grapheme_cluster: "😊".to_string(),
//                 },
//                 // Character {
//                 //     advance_x: None,
//                 //     pixmap_index: None,
//                 //     pixmap_table_index: None,
//                 //     grapheme_cluster: "!=".to_string(),
//                 // },
//             ],
//         }];
//         font.pixmap_tables = vec![sample_pixmap_table()];
//         font.color_tables = vec![sample_color_table()];
//         font.font_tables = vec![sample_font_table()];

//         font.compact = true;
//         font
//     }

// fn sample_font_table() -> FontTable {
//     FontTable {
//         character_table_indexes: Some(vec![0]),
//         fonts: vec![Font {
//             name: "SampleToyFont".into(),
//             author: "The-Nice-One".into(),
//             version: 0,
//             font_type: FontType::Regular,
//             character_table_indexes: vec![0],
//         }],
//     }
// }

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

    process.add_required_whitespace(); // a seperate validation layer might be needed later. Although really the only character that needs fixing and only if space exists :)
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
