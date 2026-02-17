use std::collections::BTreeMap;

use write_fonts::FontBuilder;

mod cmap;
mod glyf_loca;
mod head;
mod hhea;
mod htmx;
mod maxp;
mod name;
mod os2;
mod post;

pub use cmap::*;
pub use glyf_loca::*;
pub use head::*;
pub use hhea::*;
pub use htmx::*;
pub use maxp::*;
pub use name::*;
pub use os2::*;
pub use post::*;

use crate::utilities::PixmapGlyph;

#[derive(Default, Debug)]
pub struct Process<'a> {
    pub builder: FontBuilder<'a>,

    pub units_per_em: u16,
    pub max_pixel_width: i16,
    pub max_pixel_height: i16,
    pub pixmap_pairs: BTreeMap<char, PixmapGlyph>,

    pub family_name: String,
    pub family_version: f64,
    pub target_pixel_size: i16,
    pub descender_pixels: i16,

    pub is_monospaced: bool,
}

impl Process<'_> {
    pub fn add_required_whitespace(&mut self) {
        // Check if SPACE exists
        let space_advance = if let Some(space_glyph) = self.pixmap_pairs.get(&'\u{0020}') {
            space_glyph.advance_x
        } else {
            self.max_pixel_width as u8
        };

        // Add NO-BREAK SPACE if missing
        if !self.pixmap_pairs.contains_key(&'\u{00A0}') {
            self.pixmap_pairs.insert(
                '\u{00A0}',
                PixmapGlyph {
                    advance_x: space_advance,
                    width: self.max_pixel_width as u8,
                    height: self.max_pixel_height as u8,
                    pixmap: vec![
                        false;
                        self.max_pixel_width as usize * self.max_pixel_height as usize
                    ],
                    left_side_bearing: 0,
                },
            );
        }
    }
}
