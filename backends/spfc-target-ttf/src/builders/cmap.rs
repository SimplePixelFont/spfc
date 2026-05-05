use super::Process;
use anyhow::Result;
use std::collections::BTreeMap;
use write_fonts::tables::cmap::{
    Cmap, CmapSubtable, EncodingRecord, PlatformId, SequentialMapGroup,
};

pub fn push_cmap_table(process: &mut Process) -> Result<()> {
    let char_to_glyph_id: BTreeMap<char, u16> = process.pixmap_pairs
        .iter()
        .enumerate()
        .filter_map(|(_i, (s, _))| {
            let mut cs = s.chars();
            let c = cs.next()?;
            if cs.next().is_none() {
                process.get_glyph_id(s).map(|gid| (c, gid))
            } else {
                None
            }
        })
        .collect();

    let sorted_chars: Vec<char> = char_to_glyph_id.keys().copied().collect();

    // Format 4 (BMP only, for legacy compatibility)
    let mut end_codes_f4 = vec![];
    let mut start_codes_f4 = vec![];
    let mut deltas_f4 = vec![];
    let mut offsets_f4 = vec![];

    let mut current_f4_segment: Option<(u16, u16, u16)> = None; // (start_char, end_char, start_glyph_id)

    for &ch in &sorted_chars {
        let char_code = ch as u32;
        if char_code > 0xFFFF {
            // Format 4 only handles BMP characters
            continue;
        }
        let char_code_u16 = char_code as u16;
        let glyph_id = *char_to_glyph_id.get(&ch).unwrap();

        if let Some((seg_start_char, seg_end_char, seg_start_glyph_id)) = current_f4_segment {
            if char_code_u16 == seg_end_char + 1
                && glyph_id == seg_start_glyph_id + (char_code_u16 - seg_start_char)
            {
                // Extend current segment
                current_f4_segment = Some((seg_start_char, char_code_u16, seg_start_glyph_id));
            } else {
                // End previous segment and start a new one
                end_codes_f4.push(seg_end_char);
                start_codes_f4.push(seg_start_char);
                let delta = seg_start_glyph_id as i16 - seg_start_char as i16;
                deltas_f4.push(delta);
                offsets_f4.push(0); // Use delta formula

                current_f4_segment = Some((char_code_u16, char_code_u16, glyph_id));
            }
        } else {
            // Start the first segment
            current_f4_segment = Some((char_code_u16, char_code_u16, glyph_id));
        }
    }

    // Push the last Format 4 segment if any
    if let Some((seg_start_char, seg_end_char, seg_start_glyph_id)) = current_f4_segment {
        end_codes_f4.push(seg_end_char);
        start_codes_f4.push(seg_start_char);
        let delta = seg_start_glyph_id as i16 - seg_start_char as i16;
        deltas_f4.push(delta);
        offsets_f4.push(0); // Use delta formula
    }

    // Terminator segment for Format 4
    end_codes_f4.push(0xFFFF);
    start_codes_f4.push(0xFFFF);
    deltas_f4.push(1); // Maps 0xFFFF to glyph 0 (notdef)
    offsets_f4.push(0);
    let format_4_subtable = CmapSubtable::format_4(
        0,
        end_codes_f4,
        start_codes_f4,
        deltas_f4,
        offsets_f4,
        vec![],
    );

    // Format 12 (Full Unicode range, required for emojis)
    let mut groups_f12 = vec![];
    let mut current_f12_segment: Option<(u32, u32, u32)> = None; // (start_char, end_char, start_glyph_id)

    for &ch in &sorted_chars {
        let char_code = ch as u32;
        let glyph_id = *char_to_glyph_id.get(&ch).unwrap() as u32;

        if let Some((seg_start_char, seg_end_char, seg_start_glyph_id)) = current_f12_segment {
            if char_code == seg_end_char + 1
                && glyph_id == seg_start_glyph_id + (char_code - seg_start_char)
            {
                // Extend current segment
                current_f12_segment = Some((seg_start_char, char_code, seg_start_glyph_id));
            } else {
                // End previous segment and start a new one
                groups_f12.push(SequentialMapGroup::new(
                    seg_start_char,
                    seg_end_char,
                    seg_start_glyph_id,
                ));
                current_f12_segment = Some((char_code, char_code, glyph_id));
            }
        } else {
            // Start the first segment
            current_f12_segment = Some((char_code, char_code, glyph_id));
        }
    }

    // Push the last Format 12 segment if any
    if let Some((seg_start_char, seg_end_char, seg_start_glyph_id)) = current_f12_segment {
        groups_f12.push(SequentialMapGroup::new(
            seg_start_char,
            seg_end_char,
            seg_start_glyph_id,
        ));
    }
    let format_12_subtable = CmapSubtable::format_12(0, groups_f12);

    let cmap = Cmap {
        encoding_records: vec![
            // Unicode Full (Plane 0-16)
            EncodingRecord {
                platform_id: PlatformId::Unicode,
                encoding_id: 4,
                subtable: format_12_subtable.clone().into(),
            },
            // Windows Full
            EncodingRecord {
                platform_id: PlatformId::Windows,
                encoding_id: 10,
                subtable: format_12_subtable.into(),
            },
            // Legacy BMP Fallbacks
            EncodingRecord {
                platform_id: PlatformId::Unicode,
                encoding_id: 3,
                subtable: format_4_subtable.clone().into(),
            },
            EncodingRecord {
                platform_id: PlatformId::Windows,
                encoding_id: 1,
                subtable: format_4_subtable.into(),
            },
        ],
    };
    process.builder.add_table(&cmap)?;
    Ok(())
}
