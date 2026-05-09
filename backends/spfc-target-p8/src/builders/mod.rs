use std::collections::BTreeMap;

use crate::utilities::PixmapGlyph;

pub mod writer;
pub use writer::*;


#[derive(Default, Debug)]
pub struct Process {
    pub family_name: String,
    pub family_version: f64,
    pub manufacturer: String,
    pub pixmap_pairs: BTreeMap<String, PixmapGlyph>,
}