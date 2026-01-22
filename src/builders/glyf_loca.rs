use super::Process;
use anyhow::Result;
use write_fonts::tables::glyf::{GlyfLocaBuilder, Glyph, SimpleGlyph};

pub fn push_glyf_loca_tables(
    process: &mut Process
) -> Result<()> {
    let mut glyf_builder = GlyfLocaBuilder::new();

    // .notdef
    let notdef = SimpleGlyph::default();
    glyf_builder.add_glyph(&Glyph::Simple(notdef))?;

    for (_, pixmap) in &process.pixmap_pairs {
        glyf_builder.add_glyph(&pixmap.clone().into_simple_glyph(64))?;
    }

    let glyf = glyf_builder.build();
    process.builder.add_table(&glyf.0)?;
    process.builder.add_table(&glyf.1)?;
    Ok(())
}