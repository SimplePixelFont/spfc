use super::Process;
use anyhow::Result;
use write_fonts::tables::maxp::Maxp;

pub fn push_maxp_table(process: &mut Process) -> Result<()> {
    let maxp = Maxp {
        num_glyphs: process.total_glyph_count(),
        max_points: Some(process.max_points),
        max_contours: Some(process.max_contours),
        max_composite_points: Some(0),
        max_composite_contours: Some(0),
        max_zones: Some(2),
        max_twilight_points: Some(0),
        max_storage: Some(0),
        max_function_defs: Some(0),
        max_instruction_defs: Some(0),
        max_stack_elements: Some(0),
        max_size_of_instructions: Some(0),
        max_component_elements: Some(0),
        max_component_depth: Some(0),
    };
    process.builder.add_table(&maxp)?;
    Ok(())
}
