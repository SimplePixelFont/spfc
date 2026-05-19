use std::collections::BTreeMap;

use bitvec::{bitvec, field::BitField, order::Lsb0, view::BitView};
use render_spf::{
    ColorControl, RenderableTexture, cache::{TextureBuilder, generic_update_cache}
};
use spf::core::{Character, Layout, Pixmap, PixmapTable};

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
            .unwrap_or(1);

        let advance_x = character.advance_x.unwrap_or(width);
        if advance_x != width {
            todo!("ABI 0.2.0: will add logging; On p8 target the advance_x is ignored because pico-8 doesn't support it.")
        }
        if width > 8 || height > 8 {
            todo!("ABI 0.2.0: will add logging; On p8 target only 8x8 pixels are supported, Anything larger than that will be treated as 8x8.")
        }
        if bits_per_pixel != 1 {
            todo!("ABI 0.2.0: will add logging; On p8 target only 1 bit per pixel is supported, Anything other than 0 will be treated as opaque.")
        }

        let bits = pixmap.data.view_bits::<Lsb0>();
        let pixels: Vec<u8> = bits
            .chunks(bits_per_pixel as usize)
            .map(|chunk| chunk.load_be::<u8>())
            .take(width as usize * height as usize)
            .collect();

        let mut processed_pixels = Vec::with_capacity(height as usize);

        let mut current_x = 0;
        let mut current_y = 0;
        let mut pixel_row = bitvec![u8, Lsb0; 0; 8];
        for pixel in pixels {
            pixel_row.set(current_x as usize, pixel != 0);
            current_x += 1;
            if current_x == width {
                processed_pixels.push(pixel_row.load_le::<u8>());
                current_x = 0;
                current_y += 1;
                if current_y == height {
                    break;
                }
                pixel_row = bitvec![u8, Lsb0; 0; 8];
            }
        }

        PixmapGlyph {
            width,
            height,
            bitmap: processed_pixels,
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
        self.width as u32
    }
}

#[derive(Debug, Clone, Default)]
pub struct PixmapGlyph {
    pub width: u8,
    pub height: u8,
    pub bitmap: Vec<u8>,
}

pub fn create_pixmap_pairs(layout: &Layout) -> BTreeMap<String, PixmapGlyph> {
    let mut pixmap_pairs = BTreeMap::new();
    let mut color_control = ColorControl::with_capacity(layout.color_tables.len());

    generic_update_cache(
        &layout.font_tables[0],
        &layout.font_tables[0].fonts[0],
        layout,
        &PixmapGlyphTextureBuilder,
        &mut color_control,
        |grapheme| grapheme.to_string(),
        |key, glyph: PixmapGlyph| {
            pixmap_pairs.insert(key, glyph.clone());
        },
    );

    pixmap_pairs
}