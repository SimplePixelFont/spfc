use super::Process;
use crate::utilities::get_average_char_width;
use anyhow::Result;
use write_fonts::tables::os2::{Os2, SelectionFlags};

pub fn push_os2_table(process: &mut Process) -> Result<()> {
    let ascender =
        ((process.max_pixel_height - process.descender_pixels) * process.target_pixel_size) as i16;
    let descender = (process.descender_pixels * process.target_pixel_size) as i16;

    let first_char = process.pixmap_pairs.keys().min().unwrap();
    let last_char = process.pixmap_pairs.keys().max().unwrap();

    let mut os2 = Os2::default();
    os2.x_avg_char_width = get_average_char_width(&process.pixmap_pairs, process.target_pixel_size);
    os2.us_weight_class = 400;
    os2.us_width_class = 5;
    os2.us_win_ascent = ascender as u16;
    os2.us_win_descent = descender as u16;
    os2.s_typo_ascender = ascender;
    os2.s_typo_descender = -descender;
    os2.s_typo_line_gap = 0;
    os2.sx_height = Some(500);
    os2.s_cap_height = Some(700);
    os2.ul_code_page_range_1 = Some(1);
    os2.ul_code_page_range_2 = Some(0);
    os2.us_default_char = Some(0);
    os2.us_break_char = Some(32);
    os2.us_max_context = Some(0);
    os2.ul_unicode_range_1 = 1;
    os2.ul_unicode_range_2 = 0;
    os2.ul_unicode_range_3 = 0;
    os2.ul_unicode_range_4 = 0;

    os2.us_first_char_index = *first_char as u16;
    os2.us_last_char_index = *last_char as u16;

    os2.fs_type = 8; // editable embedding

    if process.is_monospaced {
        os2.panose_10 = [
            2, // bFamilyType: Latin Text
            0, // bSerifStyle: Any
            5, // bWeight: Book (400 weight)
            9, // bProportion: Monospaced ← KEY VALUE
            0, // bContrast: Any
            0, // bStrokeVariation: Any
            0, // bArmStyle: Any
            0, // bLetterform: Any
            0, // bMidline: Any
            0, // bXHeight: Any
        ];
        os2.fs_selection = SelectionFlags::REGULAR | SelectionFlags::USE_TYPO_METRICS;
    } else {
        os2.panose_10 = [
            2, // bFamilyType: Latin Text
            0, // bSerifStyle: Any
            5, // bWeight: Book (400 weight)
            3, // bProportion: Modern (proportional) ← KEY VALUE
            0, // bContrast: Any
            0, // bStrokeVariation: Any
            0, // bArmStyle: Any
            0, // bLetterform: Any
            0, // bMidline: Any
            0, // bXHeight: Any
        ];
        os2.fs_selection = SelectionFlags::REGULAR;
    }

    process.builder.add_table(&os2)?;
    Ok(())
}
