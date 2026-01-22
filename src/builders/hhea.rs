use super::Process;
use anyhow::Result;
use write_fonts::{tables::hhea::Hhea, types::{FWord, UfWord}};


pub fn push_hhea_table(process: &mut Process) -> Result<()> {
    let hhea = Hhea {
        ascender: FWord::new(process.units_per_em as i16),
        descender: FWord::new(process.decender_pixels * process.target_pixel_size),
        line_gap: FWord::new(0),
        advance_width_max: UfWord::new((process.max_pixel_width + 1) as u16 * process.target_pixel_size as u16), // Add one for default spacing between each character
        number_of_h_metrics: process.pixmap_pairs.len() as u16+1,
        ..Default::default()
    };
    process.builder.add_table(&hhea)?;
    Ok(())
}