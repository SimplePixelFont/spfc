use anyhow::Result;
use write_fonts::tables::os2::{Os2, SelectionFlags};
use super::Process;


pub fn push_os2_table(process: &mut Process) -> Result<()> {
    let mut os2 = Os2::default();
    os2.x_avg_char_width = process.max_pixel_width * process.target_pixel_size;
    os2.us_weight_class = 400;
    os2.us_width_class = 5;
    os2.us_win_ascent = process.units_per_em as u16;
    os2.us_win_descent = process.decender_pixels as u16 * process.target_pixel_size as u16;
    os2.s_typo_ascender = process.units_per_em as i16;
    os2.s_typo_descender = process.decender_pixels as i16 * process.target_pixel_size;
    os2.s_typo_line_gap = 0;
    os2.sx_height = Some(500);
    os2.s_cap_height = Some(700);
    os2.ul_code_page_range_1 = Some(1);
    os2.ul_code_page_range_2 = Some(0);
    os2.us_default_char = Some(0);
    os2.us_break_char = Some(32);
    os2.us_max_context = Some(0);
    os2.fs_selection = SelectionFlags::REGULAR;
    os2.ul_unicode_range_1 = 1;
    os2.ul_unicode_range_2 = 0;
    os2.ul_unicode_range_3 = 0;
    os2.ul_unicode_range_4 = 0;
    process.builder.add_table(&os2)?;
    Ok(())
}
