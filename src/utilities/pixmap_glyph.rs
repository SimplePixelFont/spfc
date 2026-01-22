use std::collections::BTreeMap;

use bitvec::{field::BitField, order::Lsb0, view::BitView};
use spf::core::{Layout, Pixmap, PixmapTable};
use write_fonts::tables::glyf::SimpleGlyph;

use super::PixelGrid;

#[derive(Debug, Clone, Default)]
pub struct PixmapGlyph {
    pub advance_x: u8,
    pub width: u8,
    pub height: u8,
    pub pixmap: Vec<bool>,
}

impl PixmapGlyph {
    pub fn into_simple_glyph(self, pixel_size: u16) -> SimpleGlyph {
        let pixel_grid = PixelGrid {
            width: self.width as usize,
            height: self.height as usize,
            pixels: self
                .pixmap
                .chunks(self.width as usize)
                .map(|row| row.to_vec())
                .collect(),
            pixel_size: pixel_size as f64,
        };
        let bez_path = pixel_grid.to_bezpath();
        let glyph = SimpleGlyph::from_bezpath(&bez_path).unwrap();
        glyph
    }
}

pub fn resolve_pixmap<'a>(
    index: usize,
    pixmap_tables: &Vec<&'a PixmapTable>,
) -> Option<(&'a PixmapTable, &'a Pixmap)> {
    for pixmap_table in pixmap_tables {
        if index < pixmap_table.pixmaps.len() {
            return Some((pixmap_table, &pixmap_table.pixmaps[index]));
        }
    }
    None
}

pub fn create_pixmap_pairs(layout: &Layout) -> BTreeMap<char, PixmapGlyph> {
    let character_tables = &layout.character_tables;
    let pixmap_tables = &layout.pixmap_tables;

    let mut pixmap_pairs = BTreeMap::new();

    for character_table in character_tables {
        let dependency_pixmap_table_indexes =
            if let Some(deps) = character_table.pixmap_table_indexes.as_ref() {
                deps
            } else {
                &Vec::new()
            };

        let mut pixmaps_in_table = Vec::with_capacity(dependency_pixmap_table_indexes.len());
        for pixmap_table_index in dependency_pixmap_table_indexes {
            if let Some(pixmap_table) = pixmap_tables.get(*pixmap_table_index as usize) {
                pixmaps_in_table.push(pixmap_table);
            }
        }

        for (index, character) in character_table.characters.iter().enumerate() {
            let pixmap = if character_table.use_pixmap_index {
                let pixmap_index = character.pixmap_index.unwrap() as usize;
                resolve_pixmap(pixmap_index, &pixmaps_in_table)
            } else {
                resolve_pixmap(index, &pixmaps_in_table)
            };

            if let Some((pixmap_table, pixmap)) = pixmap {
                //println!("{:?} {:?}", pixmap_table, pixmap);
                let width = pixmap_table.constant_width.or(pixmap.custom_width).unwrap();
                let height = pixmap_table
                    .constant_height
                    .or(pixmap.custom_height)
                    .unwrap();
                let bits_per_pixel = pixmap_table
                    .constant_bits_per_pixel
                    .or(pixmap.custom_bits_per_pixel)
                    .unwrap();

                let advance_x = character.advance_x.unwrap_or(width);

                let bits = pixmap.data.view_bits::<Lsb0>();
                let pixels: Vec<u8> = bits
                    .chunks(bits_per_pixel as usize)
                    .map(|chunk| chunk.load_be::<u8>())
                    .take(width as usize * height as usize)
                    .collect();

                let mut pixel_bools = Vec::with_capacity(pixels.len());
                for byte in pixels {
                    pixel_bools.push(byte != 0);
                }

                let character = character.grapheme_cluster.chars().next().unwrap_or('\0');
                pixmap_pairs.insert(
                    character,
                    PixmapGlyph {
                        advance_x,
                        width,
                        height,
                        pixmap: pixel_bools,
                    },
                );
            }
        }
    }
    pixmap_pairs
}
