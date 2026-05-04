use anyhow::Result;
use write_fonts::{
    tables::{
        colr::{BaseGlyph, Colr, Layer},
        cpal::{ColorRecord, Cpal},
    },
    types::GlyphId16,
};

use super::Process;

pub fn push_colr_cpal_tables(process: &mut Process) -> Result<()> {
    if process.colr_base_glyph_records.is_empty() || process.cpal_color_records.is_empty() {
        return Ok(());
    }

    let cpal_records: Vec<ColorRecord> = process
        .cpal_color_records
        .iter()
        .map(|color| ColorRecord::new(color.b, color.g, color.r, color.a))
        .collect();

    let cpal = Cpal::new(
        cpal_records.len() as u16,
        1,
        cpal_records.len() as u16,
        Some(cpal_records),
        vec![0],
    );
    process.builder.add_table(&cpal)?;

    let base_glyph_records: Vec<BaseGlyph> = process
        .colr_base_glyph_records
        .iter()
        .map(|record| {
            BaseGlyph::new(
                GlyphId16::new(record.glyph_id),
                record.first_layer_index,
                record.num_layers,
            )
        })
        .collect();

    let layer_records: Vec<Layer> = process
        .colr_layer_records
        .iter()
        .map(|record| Layer::new(GlyphId16::new(record.glyph_id), record.palette_index))
        .collect();

    let colr = Colr::new(
        base_glyph_records.len() as u16,
        Some(base_glyph_records),
        Some(layer_records),
        process.colr_layer_records.len() as u16,
    );
    process.builder.add_table(&colr)?;

    Ok(())
}
