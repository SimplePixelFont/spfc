use std::collections::BTreeMap;

use write_fonts::FontBuilder;

mod cmap;
mod gasp;
mod glyf_loca;
mod head;
mod hhea;
mod htmx;
mod maxp;
mod name;
mod os2;
mod post;

pub use cmap::*;
pub use gasp::*;
pub use glyf_loca::*;
pub use head::*;
pub use hhea::*;
pub use htmx::*;
pub use maxp::*;
pub use name::*;
pub use os2::*;
pub use post::*;

use crate::utilities::PixmapGlyph;

pub const AUTOINSERTED_CHARS_COUNT: u16 = 3;

#[derive(Default, Debug)]
pub struct Process<'a> {
    pub builder: FontBuilder<'a>,

    pub units_per_em: u16,
    pub max_pixel_width: i16,
    pub max_pixel_height: i16,
    pub pixmap_pairs: BTreeMap<char, PixmapGlyph>,

    pub family_name: String,
    pub family_version: f64,
    pub copyright: String,
    pub manufacturer: String,
    pub vendor_url: String,
    pub license_description: String,
    pub target_pixel_size: i16,
    pub descender_pixels: i16,

    pub is_monospaced: bool,
    pub max_points: u16,
    pub max_contours: u16,
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
    pub fn update_max_points_and_contours(&mut self) {
        for (_, pixmap) in &self.pixmap_pairs {
            let glyph = pixmap.clone().into_simple_glyph(
                self.target_pixel_size as u16,
                self.descender_pixels as usize,
            );

            let mut points = 0;
            for countor in &glyph.contours {
                points += countor.iter().count() as u16
            }
            let countors = glyph.contours.len() as u16;
            self.max_points = self.max_points.max(points);
            self.max_contours = self.max_contours.max(countors);
        }
    }
}
