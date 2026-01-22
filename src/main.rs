use spf::core::layout_from_data;

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
    process.target_pixel_size = args.pixel_size;
    process.decender_pixels = args.decender_pixels;

    process.pixmap_pairs = create_pixmap_pairs(&layout);
    process.max_pixel_width = max_width(&process.pixmap_pairs);
    process.max_pixel_height = max_height(&process.pixmap_pairs);
    process.units_per_em = calculate_units_per_em(
        process.max_pixel_width,
        process.max_pixel_height,
        process.target_pixel_size,
    );

    push_head_table(&mut process)?;
    push_hhea_table(&mut process)?;
    push_maxp_table(&mut process)?;
    push_os2_table(&mut process)?;
    push_name_table(&mut process)?;
    push_post_table(&mut process)?;
    push_glyf_loca_tables(&mut process)?;
    push_hmtx_table(&mut process)?;
    push_cmap_table(&mut process)?;

    let font_data = process.builder.build();
    std::fs::write(&args.output, &font_data)?;

    println!("Finished writing {} bytes to {}", font_data.len(), args.output);

    Ok(())
}
