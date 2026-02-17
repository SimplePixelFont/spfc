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
