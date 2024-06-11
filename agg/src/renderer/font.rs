use crate::renderer::{color_to_rgb, text_attrs, Renderer, Settings};
use crate::theme::Theme;
use crate::vt::Frame;
use avt::Color;
use fontdue::{Font, FontSettings, Metrics};
use imgref::ImgVec;
use resvg::usvg::fontdb::{Database, Family, Query, Stretch, Style, Weight};
use rgb::RGBA8;
use std::collections::HashMap;

fn get_font(db: &Database, families: &[String], weight: Weight, style: Style) -> Option<Font> {
    log::debug!(
        "looking up font for families={:?}, weight={}, style={:?}",
        families,
        weight.0,
        style
    );

    let families: Vec<Family> = families.iter().map(|name| Family::Name(name)).collect();

    let query = Query {
        families: &families,
        stretch: Stretch::Normal,
        weight,
        style,
    };
    let font_id = db.query(&query)?;
    log::debug!("found font with id={:?}", font_id);

    db.with_face_data(font_id, |font_data, face_index| {
        let settings = FontSettings {
            collection_index: face_index,
            ..Default::default()
        };
        Font::from_bytes(font_data, settings).unwrap()
    })
}

type FontFace = (String, bool, bool);
type CharVariant = (char, bool, bool);
type Glyph = (Metrics, Vec<u8>);

#[derive(Debug)]
pub struct FontRenderer {
    font_families: Vec<String>,
    theme: Theme,
    pixel_width: usize,
    pixel_height: usize,
    font_size: usize,
    col_width: f64,
    row_height: f64,
    font_db: Database,
    font_cache: HashMap<FontFace, Option<Font>>,
    glyph_cache: HashMap<CharVariant, Option<Glyph>>,
}

impl FontRenderer {
    pub fn new(settings: Settings) -> Self {
        let default_font = get_font(
            &settings.font_db,
            &settings.font_families,
            Weight::NORMAL,
            Style::Normal,
        )
        .unwrap();

        let metrics = default_font.metrics('/', settings.font_size as f32);
        let (cols, rows) = settings.terminal_size;
        let col_width = metrics.advance_width as f64;
        let row_height = (settings.font_size as f64) * settings.line_height;

        Self {
            font_families: settings.font_families,
            theme: settings.theme,
            pixel_width: ((cols + 2) as f64 * col_width).round() as usize,
            pixel_height: ((rows + 1) as f64 * row_height).round() as usize,
            font_size: settings.font_size,
            col_width,
            row_height,
            font_db: settings.font_db,
            font_cache: HashMap::new(),
            glyph_cache: HashMap::new(),
        }
    }

    fn get_glyph(&self, ch: char, bold: bool, italic: bool) -> &Option<Glyph> {
        self.glyph_cache.get(&(ch, bold, italic)).unwrap()
    }

    fn ensure_glyph(&mut self, ch: char, bold: bool, italic: bool) {
        let key = (ch, bold, italic);
        if self.glyph_cache.contains_key(&key) {
            return;
        }
        let rasterized = match self.rasterize_glyph(ch, bold, italic) {
            Some(glyph) => Some(glyph),
            None => {
                if bold || italic {
                    self.rasterize_glyph(ch, false, false)
                } else {
                    None
                }
            }
        };
        self.glyph_cache.insert(key, rasterized);
    }

    fn rasterize_glyph(&mut self, ch: char, bold: bool, italic: bool) -> Option<Glyph> {
        let font_size = self.font_size as f32;
        self.font_families.clone().into_iter().find_map(|name| {
            match self.get_font(name, bold, italic) {
                Some(font) => {
                    let idx = font.lookup_glyph_index(ch);
                    if idx > 0 {
                        Some(font.rasterize_indexed(idx, font_size))
                    } else {
                        None
                    }
                }
                None => None,
            }
        })
    }

    fn get_font(&mut self, name: String, bold: bool, italic: bool) -> &Option<Font> {
        self.font_cache
            .entry((name.clone(), bold, italic))
            .or_insert_with(|| {
                let weight = if bold { Weight::BOLD } else { Weight::NORMAL };
                let style = if italic { Style::Italic } else { Style::Normal };
                get_font(&self.font_db, &[name], weight, style)
            })
    }
}

fn mix_colors(fg: RGBA8, bg: RGBA8, ratio: u8) -> RGBA8 {
    let ratio = ratio as u16;
    RGBA8::new(
        ((bg.r as u16) * (255 - ratio) / 255) as u8 + ((fg.r as u16) * ratio / 255) as u8,
        ((bg.g as u16) * (255 - ratio) / 255) as u8 + ((fg.g as u16) * ratio / 255) as u8,
        ((bg.b as u16) * (255 - ratio) / 255) as u8 + ((fg.b as u16) * ratio / 255) as u8,
        255,
    )
}

impl Renderer for FontRenderer {
    fn render(&mut self, frame: Frame) -> ImgVec<RGBA8> {
        let initial_color = self.theme.background.alpha(255);
        let mut buf: Vec<RGBA8> = vec![initial_color; self.pixel_width * self.pixel_height];

        let margin_l = self.col_width;
        let margin_t = (self.row_height / 2.0).round() as usize;

        let mut foregrounds: HashMap<usize, RGBA8> = HashMap::default();

        for (row, chars) in frame.lines.iter().enumerate() {
            let y_t = margin_t + (row as f64 * self.row_height).round() as usize;
            let y_b = margin_t + ((row + 1) as f64 * self.row_height).round() as usize;

            for (col, (_, pen)) in chars.iter().enumerate() {
                let x_l = (margin_l + col as f64 * self.col_width).round() as usize;
                let x_r = (margin_l + (col + 1) as f64 * self.col_width).round() as usize;

                let attrs = text_attrs(pen, &frame.cursor, col, row, &self.theme);

                let fg_color = attrs
                    .foreground
                    .unwrap_or(Color::RGB(self.theme.foreground));
                let fg = color_to_rgb(&fg_color, &self.theme).alpha(255);

                if let Some(bg_color) = attrs.background {
                    let bg = color_to_rgb(&bg_color, &self.theme);
                    for y in y_t..y_b {
                        for x in x_l..x_r {
                            let idx = y * self.pixel_width + x;
                            buf[idx] = bg.alpha(255);
                            foregrounds.insert(idx, fg);
                        }
                    }
                }

                if pen.is_underline() {
                    let y = margin_t
                        + (row as f64 * self.row_height + self.font_size as f64 * 1.2).round()
                            as usize;
                    for x in x_l..x_r {
                        let idx = y * self.pixel_width + x;
                        buf[idx] = fg;
                    }
                }
            }

            for (col, (ch, pen)) in chars.iter().enumerate() {
                let x_l = (margin_l + col as f64 * self.col_width).round() as usize;

                if ch == &' ' {
                    continue;
                }

                self.ensure_glyph(*ch, pen.is_bold(), pen.is_italic());
                let glyph = self.get_glyph(*ch, pen.is_bold(), pen.is_italic());
                if glyph.is_none() {
                    continue;
                }
                let (metrics, bitmap) = glyph.as_ref().unwrap();

                let y_offset = (y_t + self.font_size - metrics.height) as i32 - metrics.ymin;
                let x_offset = x_l as i32 + metrics.xmin;

                for bmap_y in 0..metrics.height {
                    let y = y_offset + bmap_y as i32;
                    if y < 0 || y >= self.pixel_height as i32 {
                        continue;
                    }
                    for bmap_x in 0..metrics.width {
                        let x = x_offset + bmap_x as i32;
                        if x < 0 || x >= self.pixel_width as i32 {
                            continue;
                        }
                        // FIX KEY ERROR
                        let idx = (y as usize) * self.pixel_width + (x as usize);
                        let fg = foregrounds[&idx];
                        let bg = buf[idx];
                        let v = bitmap[bmap_y * metrics.width + bmap_x];
                        buf[idx] = mix_colors(fg, bg, v);
                    }
                }
            }
        }

        ImgVec::new(buf, self.pixel_width, self.pixel_height)
    }

    fn pixel_size(&self) -> (usize, usize) {
        (self.pixel_width, self.pixel_height)
    }
}
