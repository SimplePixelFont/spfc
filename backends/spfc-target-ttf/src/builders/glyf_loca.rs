use crate::utilities::PixelGrid;

use super::Process;
use anyhow::Result;
use kurbo::BezPath;
use write_fonts::tables::glyf::{GlyfLocaBuilder, SimpleGlyph};

pub fn push_glyf_loca_tables(process: &mut Process) -> Result<()> {
    let mut glyf_builder = GlyfLocaBuilder::new();

    // .notdef - temp. solution because SPF has not standardized "special characters"
    let mut notdef = PixelGrid::empty(
        process.max_pixel_width as usize,
        process.max_pixel_height as usize,
        process.target_pixel_size as f64,
    );
    notdef.draw_rectangle(
        0,
        0,
        process.max_pixel_width as usize - 1,
        process.max_pixel_height as usize - process.descender_pixels as usize - 1,
    );
    let notdef =
        SimpleGlyph::from_bezpath(&notdef.to_bezpath(process.descender_pixels as usize)).unwrap();
    glyf_builder.add_glyph(&notdef)?;

    let null_glyph = SimpleGlyph::from_bezpath(&BezPath::new()).unwrap();
    glyf_builder.add_glyph(&null_glyph)?;

    // Glyph 2: nonmarkingreturn (empty glyph for tab/return)
    let nonmarkingreturn = SimpleGlyph::from_bezpath(&BezPath::new()).unwrap();
    glyf_builder.add_glyph(&nonmarkingreturn)?;

    for (_, pixmap) in &process.pixmap_pairs {
        let glyph = pixmap.clone().into_simple_glyph(
            process.target_pixel_size as u16,
            process.descender_pixels as usize,
        );
        glyf_builder.add_glyph(&glyph)?;
    }

    let glyf = glyf_builder.build();
    process.builder.add_table(&glyf.0)?;
    process.builder.add_table(&glyf.1)?;
    Ok(())
}
