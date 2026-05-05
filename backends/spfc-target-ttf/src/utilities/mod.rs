mod pixel_grid;
use std::collections::BTreeMap;

pub use pixel_grid::*;

mod pixmap_glyph;
pub use pixmap_glyph::*;

pub fn calculate_units_per_em(grid_width: i16, grid_height: i16, target_pixel_size: i16) -> u16 {
    let max_dimension = grid_width.max(grid_height);
    (max_dimension * target_pixel_size) as u16
}

pub fn max_width(pixmap_glyphs: &BTreeMap<String, PixmapGlyph>) -> i16 {
    pixmap_glyphs
        .iter()
        .map(|pair| pair.1.width as usize)
        .max()
        .unwrap_or(0) as i16
}
pub fn max_height(pixmap_glyphs: &BTreeMap<String, PixmapGlyph>) -> i16 {
    pixmap_glyphs
        .iter()
        .map(|pair| pair.1.height as usize)
        .max()
        .unwrap_or(0) as i16
}
pub fn is_monospaced(pixmap_glyphs: &BTreeMap<String, PixmapGlyph>) -> bool {
    if pixmap_glyphs.is_empty() {
        return false;
    }

    let first_advance = pixmap_glyphs.values().next().map(|g| g.advance_x).unwrap_or(0);
    pixmap_glyphs
        .values()
        .all(|glyph| glyph.advance_x == first_advance)
}

pub fn get_average_char_width(pixmap_glyphs: &BTreeMap<String, PixmapGlyph>, target_pixel_size: i16) -> i16 {
    if pixmap_glyphs.is_empty() {
        return 0;
    }

    let total_advance: i32 = pixmap_glyphs
        .values()
        .map(|glyph| (glyph.advance_x+1) as i32 * target_pixel_size as i32)
        .sum();
    (total_advance / pixmap_glyphs.len() as i32) as i16
}