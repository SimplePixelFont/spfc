use super::Process;
use anyhow::Result;
use write_fonts::tables::cmap::{Cmap, CmapSubtable, EncodingRecord, PlatformId};

fn group_into_segments(chars: &[char]) -> Vec<(u16, u16)> {
    if chars.is_empty() {
        return vec![];
    }

    let mut segments = vec![];
    let mut start = chars[0] as u16;
    let mut end = start;

    for &ch in &chars[1..] {
        let code = ch as u16;
        if code == end + 1 {
            end = code;
        } else {
            segments.push((start, end));
            start = code;
            end = code;
        }
    }

    segments.push((start, end));

    segments
}

pub fn push_cmap_table(process: &mut Process) -> Result<()> {
    let sorted_chars: Vec<char> = process.pixmap_pairs.keys().copied().collect();
    let segments = group_into_segments(&sorted_chars);

    let mut end_codes = vec![];
    let mut start_codes = vec![];
    let mut deltas = vec![];
    let mut offsets = vec![];

    let mut current_glyph_index = 1u16; // Start after .notdef

    for (start_char, end_char) in segments.iter() {
        end_codes.push(*end_char);
        start_codes.push(*start_char);

        // Calculate delta: glyph_index = char_code + delta
        // delta = glyph_index - char_code
        let delta = current_glyph_index as i16 - *start_char as i16;
        deltas.push(delta);
        offsets.push(0); // Use delta formula

        let segment_size = (*end_char - *start_char + 1) as u16;
        current_glyph_index += segment_size;
    }

    // Terminator segment
    end_codes.push(0xFFFF);
    start_codes.push(0xFFFF);
    deltas.push(1);
    offsets.push(0);

    let cmap_subtable = CmapSubtable::format_4(0, end_codes, start_codes, deltas, offsets, vec![]);

    let cmap = Cmap {
        encoding_records: vec![
            EncodingRecord {
                platform_id: PlatformId::Unicode,
                encoding_id: 4,
                subtable: cmap_subtable.clone().into(),
            },
            EncodingRecord {
                platform_id: PlatformId::Windows,
                encoding_id: 1,
                subtable: cmap_subtable.into(),
            },
        ],
    };
    process.builder.add_table(&cmap)?;
    Ok(())
}
