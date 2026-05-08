use std::collections::BTreeMap;

use crate::utilities::PixmapGlyph;


#[derive(Default, Debug)]
pub struct Process {
    pub family_name: String,
    pub family_version: f64,
    pub manufacturer: String,
    pub pixmap_pairs: BTreeMap<char, PixmapGlyph>,
}