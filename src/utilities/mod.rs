mod pixel_grid;
use std::collections::BTreeMap;

pub use pixel_grid::*;

mod pixmap_glyph;
pub use pixmap_glyph::*;

pub fn calculate_units_per_em(grid_width: i16, grid_height: i16, target_pixel_size: i16) -> u16 {
    let max_dimension = grid_width.max(grid_height);
    (max_dimension * target_pixel_size) as u16
}

pub fn max_width(pixmap_glyphs: &BTreeMap<char, PixmapGlyph>) -> i16 {
    pixmap_glyphs.iter().map(|pair| pair.1.width as usize).max().unwrap_or(0) as i16
}

pub fn max_height(pixmap_glyphs: &BTreeMap<char, PixmapGlyph>) -> i16 {
    pixmap_glyphs.iter().map(|pair| pair.1.height as usize).max().unwrap_or(0) as i16
}