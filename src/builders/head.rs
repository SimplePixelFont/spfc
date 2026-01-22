use write_fonts::tables::head::{Head, MacStyle};
use super::Process;
use anyhow::Result;

pub fn push_head_table(process: &mut Process) -> Result<()> {
    let head = Head {
        units_per_em: process.units_per_em,
        x_min: 0,
        y_min: 0,
        x_max: process.max_pixel_width*process.target_pixel_size,
        y_max: process.units_per_em as i16,
        mac_style: MacStyle::empty(),
        ..Default::default()
    };
    process.builder.add_table(&head)?;
    Ok(())
}