use spf::core::{layout_from_data, layout_to_data};

mod builders;
use builders::*;

mod utilities;
use utilities::*;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input SimplePixelFont file path
    #[arg(short, long)]
    input: String,

    /// Output TTF file path
    #[arg(short, long, default_value_t = String::from("output.ttf"))]
    output: String,

    /// Name of the font family
    #[arg(short, long, default_value_t = String::from("SimplePixelFont"))]
    family_name: String,

    /// Description of the font copyright
    #[arg(short, long, default_value_t = String::from("Copyright (c) 2026 SimplePixelFont"))]
    copyright: String,

    /// Name of the font manufacturer
    #[arg(short, long, default_value_t = String::from("SimplePixelFont"))]
    manufacturer: String,

    /// URL of the font vendor
    #[arg(short, long, default_value_t = String::from("https://github.com/SimplePixelFont"))]
    vendor_url: String,

    /// Description of the font license
    #[arg(short, long, default_value_t = String::from("Licensed under the Apache License, Version 2.0; you may not use this file except in compliance with the License. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0"))]
    license_description: String,

    /// Version of the font family
    #[arg(short, long, default_value_t = 1.00)]
    family_version: f64,

    /// Pixel size in font units
    #[arg(short, long, default_value_t = 64)]
    pixel_size: i16,

    /// Decender size in pixels
    #[arg(short, long, default_value_t = 0)]
    decender_pixels: i16,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let data = std::fs::read(&args.input).unwrap();
    let layout = layout_from_data(&data).unwrap();

    let mut process = Process::default();
    process.family_name = args.family_name;
    process.family_version = args.family_version;
    process.copyright = args.copyright;
    process.manufacturer = args.manufacturer;
    process.vendor_url = args.vendor_url;
    process.license_description = args.license_description;
    process.target_pixel_size = args.pixel_size;
    process.descender_pixels = args.decender_pixels;

    process.pixmap_pairs = create_pixmap_pairs(&layout);
    process.max_pixel_width = max_width(&process.pixmap_pairs);
    process.max_pixel_height = max_height(&process.pixmap_pairs);
    process.units_per_em = calculate_units_per_em(
        process.max_pixel_width,
        process.max_pixel_height,
        process.target_pixel_size,
    );

    process.add_required_whitespace(); // a seperate validation layer might be needed later. Although really the only character that needs fixing and only if space exists :)
    process.update_max_points_and_contours();
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

    process.builder.add_raw(
        write_fonts::types::Tag::new(b"prep"),
        vec![0xB8, 0x01, 0xFF, 0x85, 0xB0, 0x04, 0x8D],
    );

    // store raw spf data in a custom table for potential future use. 
    // This is not necessary for the font to function, but it allows the original data to be preserved inside the font file itself, 
    // which can be useful to debug.
    process.builder.add_raw(
        write_fonts::types::Tag::new(b"bspf"),
        layout_to_data(&layout).unwrap(),
    );

    let font_data = process.builder.build();
    std::fs::write(&args.output, &font_data)?;

    println!(
        "Finished writing {} bytes to {}",
        font_data.len(),
        args.output
    );

    Ok(())
}
