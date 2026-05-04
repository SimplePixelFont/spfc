use std::collections::BTreeMap;

use bitvec::{field::BitField, order::Lsb0, view::BitView};
use render_spf::{
    ColorControl, PixelRef, RenderableTexture, cache::{TextureBuilder, generic_update_cache}
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
        layout: &Layout,
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
            .expect("no bits_per_pixel defined in pixmap or pixmap table");

        let advance_x = character.advance_x.unwrap_or(width);
        let color_table_index = pixmap_table
            .color_table_indexes
            .as_ref()
            .and_then(|indexes| indexes.first().copied());

        let bits = pixmap.data.view_bits::<Lsb0>();
        let mut pixels = Vec::with_capacity(width as usize * height as usize);
        let mut opaque_mask = Vec::with_capacity(width as usize * height as usize);

        for chunk in bits
            .chunks(bits_per_pixel as usize)
            .take(width as usize * height as usize)
        {
            let color_index = chunk.load_be::<u8>();
            let table_idx = color_table_index.unwrap_or(0);

            let is_opaque = layout
                .color_tables
                .get(table_idx as usize)
                .and_then(|color_table| {
                    color_table.colors.get(color_index as usize).map(|color| {
                        color_table
                            .constant_alpha
                            .or(color.custom_alpha)
                            .unwrap_or(255)
                            > 0
                    })
                })
                .unwrap_or(false);

            pixels.push(PixelRef {
                color_table_index: table_idx,
                color_index,
            });
            opaque_mask.push(is_opaque);
        }
        let left_side_bearing =
            calculate_left_bearing(&opaque_mask, width as usize, height as usize);

        PixmapGlyph {
            advance_x,
            width,
            height,
            pixels,
            opaque_mask,
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
    pub pixels: Vec<PixelRef>,
    pub opaque_mask: Vec<bool>,
    pub left_side_bearing: i16,
}

impl PixmapGlyph {
    pub fn into_simple_glyph(self, pixel_size: u16, descender_pixels: usize) -> SimpleGlyph {
        glyph_from_mask(
            self.width,
            self.height,
            &self.opaque_mask,
            pixel_size,
            descender_pixels,
        )
    }

    pub fn from_mask(source: &PixmapGlyph, mask: Vec<bool>) -> Self {
        let left_side_bearing =
            calculate_left_bearing(&mask, source.width as usize, source.height as usize);
        Self {
            advance_x: source.advance_x,
            width: source.width,
            height: source.height,
            pixels: Vec::new(),
            opaque_mask: mask,
            left_side_bearing,
        }
    }

    pub fn used_pixel_refs(&self) -> Vec<PixelRef> {
        let mut refs = self
            .pixels
            .iter()
            .enumerate()
            .filter(|(i, _)| self.opaque_mask.get(*i).copied().unwrap_or(false))
            .map(|(_, &r)| r)
            .collect::<Vec<_>>();

        refs.sort_by_key(|r| (r.color_table_index, r.color_index));
        refs.dedup();
        refs
    }

    pub fn mask_for_pixel_ref(&self, pixel_ref: PixelRef) -> Vec<bool> {
        self.pixels
            .iter()
            .enumerate()
            .map(|(index, &r)| {
                self.opaque_mask.get(index).copied().unwrap_or(false)
                    && r.color_table_index == pixel_ref.color_table_index
                    && r.color_index == pixel_ref.color_index
            })
            .collect()
    }
}

fn glyph_from_mask(
    width: u8,
    height: u8,
    mask: &[bool],
    pixel_size: u16,
    descender_pixels: usize,
) -> SimpleGlyph {
    let pixel_grid = PixelGrid {
        width: width as usize,
        height: height as usize,
        pixels: mask
            .chunks(width as usize)
            .map(|row| row.to_vec())
            .collect(),
        pixel_size: pixel_size as f64,
    };
    let bez_path = pixel_grid.to_bezpath(descender_pixels);
    SimpleGlyph::from_bezpath(&bez_path).unwrap()
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
