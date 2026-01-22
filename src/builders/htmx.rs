use super::Process;
use anyhow::Result;
use write_fonts::tables::{hmtx::Hmtx, vmtx::LongMetric};

pub fn push_hmtx_table(
    process: &mut Process,
) -> Result<()> {
    let mut hmtx_metrics = vec![];
    let letter_spacing = process.target_pixel_size as u16;

    // .notdef
    hmtx_metrics.push(LongMetric { advance: process.max_pixel_width as u16 * process.target_pixel_size as u16 + letter_spacing, side_bearing: 0 });

    for (_, pixmap) in &process.pixmap_pairs {
        hmtx_metrics.push(LongMetric { advance: pixmap.advance_x as u16 *process.target_pixel_size as u16 + letter_spacing, side_bearing: 0 });
    }

    let hmtx = Hmtx {
        h_metrics: hmtx_metrics,
        left_side_bearings: vec![],
    };
    process.builder.add_table(&hmtx)?;
    Ok(())
}