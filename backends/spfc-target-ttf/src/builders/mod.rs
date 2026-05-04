use std::collections::BTreeMap;
use render_spf::PixelRef;
use spf::core::Layout;

use write_fonts::FontBuilder;

mod cmap;
mod colr_cpal;
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
pub use colr_cpal::*;
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
#[derive(Default, Debug, Clone)]
pub struct CpalColorRecordData {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Default, Debug, Clone)]
pub struct ColrBaseGlyphRecordData {
    pub glyph_id: u16,
    pub first_layer_index: u16,
    pub num_layers: u16,
}

#[derive(Default, Debug, Clone)]
pub struct ColrLayerRecordData {
    pub glyph_id: u16,
    pub palette_index: u16,
}

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

    pub cpal_color_records: Vec<CpalColorRecordData>,
    pub colr_base_glyph_records: Vec<ColrBaseGlyphRecordData>,
    pub colr_layer_records: Vec<ColrLayerRecordData>,
    pub color_layer_glyphs: Vec<PixmapGlyph>,
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
                    pixels: vec![
                        PixelRef::default();
                        self.max_pixel_width as usize * self.max_pixel_height as usize
                    ],
                    opaque_mask: vec![
                        false;
                        self.max_pixel_width as usize * self.max_pixel_height as usize
                    ],
                    left_side_bearing: 0,
                },
            );
        }
    }

    pub fn prepare_color_font_data(&mut self, layout: &Layout) {
        self.cpal_color_records.clear();
        self.colr_base_glyph_records.clear();
        self.colr_layer_records.clear();
        self.color_layer_glyphs.clear();

        if layout.color_tables.is_empty() {
            return;
        }

        let mut palette_base_offsets = vec![0u16; layout.color_tables.len()];

        for (table_index, color_table) in layout.color_tables.iter().enumerate() {
            let Some(offset) = u16::try_from(self.cpal_color_records.len()).ok() else {
                break;
            };
            palette_base_offsets[table_index] = offset;

            for color in &color_table.colors {
                self.cpal_color_records.push(CpalColorRecordData {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: color_table.constant_alpha.or(color.custom_alpha).unwrap_or(255),
                });
            }
        }

        if self.cpal_color_records.is_empty() {
            return;
        }

        let base_glyph_count = self.pixmap_pairs.len() as u16;
        let mut next_layer_glyph_id = AUTOINSERTED_CHARS_COUNT + base_glyph_count;

        for (base_index, (_, pixmap)) in self.pixmap_pairs.iter().enumerate() {
            let used_refs = pixmap.used_pixel_refs();

            // Optimization: If a glyph uses exactly one opaque color, and that color
            // matches the font's primary palette entry (Table 0, Index 1), 
            // we skip creating COLR layers. The renderer will automatically use 
            // the monochrome silhouette fallback (base glyph) in the default text color.
            if used_refs.len() == 1 {
                let r = used_refs[0];
                if r.color_table_index == 0 && r.color_index == 1 {
                    continue;
                }
            }

            let first_layer_index = self.colr_layer_records.len() as u16;

            for pixel_ref in used_refs {
                let table_idx = pixel_ref.color_table_index as usize;
                let color_idx = pixel_ref.color_index as usize;

                let Some(color_table) = layout.color_tables.get(table_idx) else {
                    continue;
                };
                let alpha = color_table.colors.get(color_idx)
                    .map(|c| color_table.constant_alpha.or(c.custom_alpha).unwrap_or(255))
                    .unwrap_or(0);

                if alpha == 0 {
                    continue;
                }

                let Some(palette_index) = palette_base_offsets.get(table_idx)
                    .and_then(|&offset| offset.checked_add(color_idx as u16)) else {
                    continue;
                };

                let layer_mask = pixmap.mask_for_pixel_ref(pixel_ref);
                if !layer_mask.iter().any(|pixel| *pixel) {
                    continue;
                }

                self.color_layer_glyphs
                    .push(PixmapGlyph::from_mask(pixmap, layer_mask));
                self.colr_layer_records.push(ColrLayerRecordData {
                    glyph_id: next_layer_glyph_id,
                    palette_index,
                });
                next_layer_glyph_id = next_layer_glyph_id.saturating_add(1);
            }

            let num_layers =
                self.colr_layer_records.len() as u16 - first_layer_index;
            if num_layers > 0 {
                self.colr_base_glyph_records.push(ColrBaseGlyphRecordData {
                    glyph_id: AUTOINSERTED_CHARS_COUNT + base_index as u16,
                    first_layer_index,
                    num_layers,
                });
            }
        }
    }

    pub fn total_glyph_count(&self) -> u16 {
        let dynamic_glyphs = self
            .pixmap_pairs
            .len()
            .saturating_add(self.color_layer_glyphs.len());
        let dynamic_glyphs = u16::try_from(dynamic_glyphs).unwrap_or(u16::MAX);
        AUTOINSERTED_CHARS_COUNT.saturating_add(dynamic_glyphs)
    }
    pub fn update_max_points_and_contours(&mut self) {
        self.max_points = 0;
        self.max_contours = 0;

        for pixmap in self
            .pixmap_pairs
            .values()
            .chain(self.color_layer_glyphs.iter())
        {
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
