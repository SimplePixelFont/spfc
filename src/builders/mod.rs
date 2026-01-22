use std::collections::BTreeMap;

use write_fonts::FontBuilder;

mod cmap;
mod head;
mod hhea;
mod maxp;
mod os2;
mod name;
mod post;
mod glyf_loca;
mod htmx;

pub use cmap::*;
pub use head::*;
pub use hhea::*;
pub use maxp::*;
pub use os2::*;
pub use name::*;
pub use post::*;
pub use glyf_loca::*;
pub use htmx::*;

use crate::utilities::PixmapGlyph;

#[derive(Default, Debug)]
pub struct Process<'a> {
    pub builder: FontBuilder<'a>,
    
    pub units_per_em: u16,
    pub max_pixel_width: i16,
    pub max_pixel_height: i16,
    pub pixmap_pairs: BTreeMap<char, PixmapGlyph>,
    
    pub family_name: String,
    pub target_pixel_size: i16,
    pub decender_pixels: i16,
}