
pub mod pixmap;
use std::collections::BTreeMap;

pub use pixmap::*;

pub mod characters;
pub use characters::*;

pub fn max_width(bitmaps: &BTreeMap<String, PixmapGlyph>) -> u8 {
    bitmaps
        .iter()
        .map(|pair| pair.1.width as usize)
        .max()
        .unwrap_or(0) as u8
}

pub fn max_height(bitmaps: &BTreeMap<String, PixmapGlyph>) -> u8 {
    bitmaps
        .iter()
        .map(|pair| pair.1.height as usize)
        .max()
        .unwrap_or(0) as u8
}

pub fn last_character_index(bitmaps: &BTreeMap<String, PixmapGlyph>) -> u8 {
    bitmaps
        .iter()
        .map(|pair| {
            if let Some(p8_char) = get_character_by_symbol(pair.0) {
                p8_char.id as usize
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0) as u8
}