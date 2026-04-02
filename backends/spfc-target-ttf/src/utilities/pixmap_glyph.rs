use std::collections::BTreeMap;

use bitvec::{field::BitField, order::Lsb0, view::BitView};
use render_spf::{
    ColorControl, RenderableTexture, cache::{TextureBuilder, generic_update_cache}
};
use spf::core::{Character, Layout, Pixmap, PixmapTable};
use write_fonts::tables::glyf::SimpleGlyph;

use super::PixelGrid;

pub(crate) struct PixmapGlyphTextureBuilder;

impl TextureBuilder<PixmapGlyph> for PixmapGlyphTextureBuilder {
    fn build_texture(
        &self,
        character: &Character,
        pixmap: &Pixmap,
        pixmap_table: &PixmapTable,
        _layout: &Layout,
    ) -> PixmapGlyph {
        let width = pixmap_table
            .constant_width
            .or(pixmap.custom_width)
            .expect("no width defined in pixmap or pixmap table");
        let height = pixmap_table
            .constant_height
            .or(pixmap.custom_height)
            .expect("no height defined in pixmap or pixmap table");

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

        let left_side_bearing =
            calculate_left_bearing(&pixel_bools, width as usize, height as usize);
        PixmapGlyph {
            advance_x,
            width,
            height,
            pixmap: pixel_bools,
            left_side_bearing,
        }
    }
}

impl RenderableTexture for PixmapGlyph {
    fn width(&self) -> u32 {
        self.width as u32
    }
    fn height(&self) -> u32 {
        self.height as u32
    }
    fn advance_x(&self) -> u32 {
        self.advance_x as u32
    }
}

#[derive(Debug, Clone, Default)]
pub struct PixmapGlyph {
    pub advance_x: u8,
    pub width: u8,
    pub height: u8,
    pub pixmap: Vec<bool>,
    pub left_side_bearing: i16,
}

impl PixmapGlyph {
    pub fn into_simple_glyph(self, pixel_size: u16, descender_pixels: usize) -> SimpleGlyph {
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
        let bez_path = pixel_grid.to_bezpath(descender_pixels);
        let glyph = SimpleGlyph::from_bezpath(&bez_path).unwrap();
        glyph
    }
}

fn calculate_left_bearing(pixels: &[bool], width: usize, height: usize) -> i16 {
    let mut empty_left_columns = 0;
    for x in 0..width {
        let mut column_empty = true;
        for y in 0..height {
            if pixels[y * width + x] {
                column_empty = false;
                break;
            }
        }
        if column_empty {
            empty_left_columns += 1;
        } else {
            break;
        }
    }

    empty_left_columns as i16
}

pub fn create_pixmap_pairs(layout: &Layout) -> BTreeMap<char, PixmapGlyph> {
    let mut pixmap_pairs = BTreeMap::new();
    let mut color_control = ColorControl::with_capacity(layout.color_tables.len());

    generic_update_cache(
        &layout.font_tables[0],
        &layout.font_tables[0].fonts[0],
        layout,
        &PixmapGlyphTextureBuilder,
        &mut color_control,
        |grapheme| grapheme.to_owned().chars().next().unwrap_or('\0'),
        |key, glyph: PixmapGlyph| {
            pixmap_pairs.insert(key, glyph.clone());
        },
    );

    pixmap_pairs
}
