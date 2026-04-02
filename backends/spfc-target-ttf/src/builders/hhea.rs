use crate::builders::AUTOINSERTED_CHARS_COUNT;

use super::Process;
use anyhow::Result;
use write_fonts::{
    tables::hhea::Hhea,
    types::{FWord, UfWord},
};

pub fn push_hhea_table(process: &mut Process) -> Result<()> {
    let ascender =
        ((process.max_pixel_height - process.descender_pixels) * process.target_pixel_size) as i16;
    let descender = (process.descender_pixels * process.target_pixel_size) as i16;

    let hhea = Hhea {
        ascender: FWord::new(ascender),
        descender: FWord::new(-descender),
        line_gap: FWord::new(0),
        advance_width_max: UfWord::new(
            (process.max_pixel_width + 1) as u16 * process.target_pixel_size as u16,
        ), // Add one for default spacing between each character
        number_of_h_metrics: process.pixmap_pairs.len() as u16 + AUTOINSERTED_CHARS_COUNT,
        caret_slope_rise: 1, // only for upright fonts, need to edit for italic fonts
        caret_slope_run: 0,  // same
        x_max_extent: FWord::new((process.max_pixel_width+1 * process.target_pixel_size) as i16),
        ..Default::default()
    };
    process.builder.add_table(&hhea)?;
    Ok(())
}
