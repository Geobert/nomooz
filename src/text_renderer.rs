use crate::{color_rgba::*, draw_geometry::draw_filled_circle};
use ab_glyph::{Font, FontArc, OutlinedGlyph, PxScale, ScaleFont};
use rust_fontconfig::{FcFontCache, FcPattern};

struct GlyphData {
    outlined: Option<OutlinedGlyph>,
    advance: f32,
    width: f32,
}

pub struct TextRenderer {
    font: FontArc,
}

impl TextRenderer {
    pub fn new(fontname: &str) -> Self {
        log::debug!("Looking for font {fontname} ...");

        let cache = FcFontCache::build();

        let mut trace = Vec::new();
        let results = cache.query_with_fallback(
            &FcPattern {
                name: Some(String::from(fontname)),
                ..Default::default()
            },
            &mut trace,
        );

        let mut font_bytes: Vec<u8> = include_bytes!("../assets/Roboto-Regular.ttf").to_vec();

        if let Some(font_match) = results {
            log::debug!("Font match ID: {:?}", font_match.id);
            log::debug!("Font unicode ranges: {:?}", font_match.unicode_ranges);

            // Get font metadata
            if let Some(meta) = cache.get_metadata_by_id(&font_match.id) {
                log::debug!("Family: {:?}", meta.family);
                log::debug!("Name: {:?}", meta.name);
            }

            // Get font file path
            if let Some(source) = cache.get_font_by_id(&font_match.id) {
                match source {
                    rust_fontconfig::OwnedFontSource::Disk(path) => {
                        log::debug!("Path: {}", path.path);
                        font_bytes = std::fs::read(path.path).expect("unable to read font file");
                    }
                    rust_fontconfig::OwnedFontSource::Memory(font) => {
                        log::debug!("Memory font: {}", font.id);
                        font_bytes = font.bytes;
                    }
                }
            }
        } else {
            log::warn!("No matching font found, falling back to the embedded font");
            font_bytes = include_bytes!("../assets/Roboto-Regular.ttf").to_vec();
        }

        let font = FontArc::try_from_vec(font_bytes).expect("invalid font");

        TextRenderer { font }
    }

    pub fn print_text(
        &self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: ColorRGBA,
        circle_color: ColorRGBA,
        canvas: &mut [u8],
        canvas_width: usize,
    ) {
        let scaled = self.font.as_scaled(PxScale::from(size));

        // Store all glyph data
        let mut glyphes: Vec<GlyphData> = Vec::new();
        text.chars().for_each(|chr| {
            let glyph_id = self.font.glyph_id(chr);
            let glyph = glyph_id.with_scale(scaled.scale());

            let glyph_data: GlyphData;

            if let Some(outlined) = scaled.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                let width = bounds.max.x - bounds.min.x;

                glyph_data = GlyphData {
                    outlined: Some(outlined),
                    advance: scaled.h_advance(glyph_id),
                    width,
                };
            } else {
                glyph_data = GlyphData {
                    outlined: None,
                    advance: scaled.h_advance(glyph_id),
                    width: 0.0,
                }
            }

            glyphes.push(glyph_data);
        });

        // Get the text width
        let width = glyphes
            .iter()
            .enumerate()
            .fold(0.0, |acc, (index, glyph_data)| {
                if index == glyphes.len() - 1 {
                    acc + glyph_data.width
                } else {
                    acc + glyph_data.advance
                }
            });

        // Horizontal offset for string centering
        let mut position_x = x - width / 2.0;

        for glyph in glyphes.iter() {
            if glyph.outlined.is_some() {
                let pen_x = position_x.round();
                let half_width = (glyph.width) / 2.0;
                draw_filled_circle(
                    (pen_x + half_width).round() as i32,
                    y as i32,
                    (size / 2.0) as i32,
                    circle_color,
                    canvas,
                    canvas_width,
                );
            }
            position_x += glyph.advance;
        }

        position_x = x - width / 2.0;

        // Really render the string
        for glyph in glyphes.iter() {
            let mut working_color = color.clone();

            if let Some(outlined) = &glyph.outlined {
                let pen_x = position_x.round();

                outlined.draw(|dx, dy, coverage| {
                    let a = (coverage * 255.0) as u8;
                    working_color.set_alpha(a);
                    let bounds = outlined.px_bounds();
                    let py = (bounds.max.y - bounds.min.y) / 2.0;

                    working_color.print_on_canvas(
                        (pen_x + dx as f32) as usize,
                        (y + dy as f32 - py).round() as usize,
                        canvas,
                        canvas_width,
                    );
                });
            }
            position_x += glyph.advance;
        }
    }
}
