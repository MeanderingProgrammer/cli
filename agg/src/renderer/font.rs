use crate::fonts::{CachingFontDb, Variant};
use crate::renderer::{text_attrs, Renderer, Settings};
use crate::theme::Theme;
use crate::vt::Frame;
use imgref::ImgVec;
use rgb::RGBA8;

#[derive(Debug)]
pub struct FontRenderer {
    font_families: Vec<String>,
    theme: Theme,
    pixel_width: usize,
    pixel_height: usize,
    font_size: usize,
    col_width: f64,
    row_height: f64,
    font_db: CachingFontDb,
}

impl FontRenderer {
    pub fn new(settings: Settings) -> Self {
        let default_font = settings
            .font_db
            .get_font(
                &settings
                    .font_families
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
                &Variant::default(),
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
        }
    }

    fn y_bounds(&self, row: usize) -> (usize, usize) {
        let margin = self.row_height / 2.0;
        let top = (margin + row as f64 * self.row_height).round() as usize;
        let bottom = (margin + (row + 1) as f64 * self.row_height).round() as usize;
        (top, bottom)
    }

    fn x_bounds(&self, col: usize) -> (usize, usize) {
        let margin = self.col_width;
        let left = (margin + col as f64 * self.col_width).round() as usize;
        let right = (margin + (col + 1) as f64 * self.col_width).round() as usize;
        (left, right)
    }
}

impl Renderer for FontRenderer {
    fn render(&mut self, frame: Frame) -> ImgVec<RGBA8> {
        let initial_color = self.theme.background.alpha(255);
        let mut buf: Vec<RGBA8> = vec![initial_color; self.pixel_width * self.pixel_height];

        // Handle the backgrounds & underlines first, ignore foreground characters
        for (row, chars) in frame.lines.iter().enumerate() {
            let (y_t, y_b) = self.y_bounds(row);
            for (col, (_, pen)) in chars.iter().enumerate() {
                let (x_l, x_r) = self.x_bounds(col);
                let attrs = text_attrs(pen, &frame.cursor, col, row, &self.theme);

                if let Some(bg) = attrs.background {
                    for y in y_t..y_b {
                        for x in x_l..x_r {
                            buf[y * self.pixel_width + x] = bg.alpha(255);
                        }
                    }
                }

                let fg = attrs.foreground.alpha(255);
                if pen.is_underline() {
                    let y = y_t + (self.font_size as f64 * 1.2).round() as usize;
                    for x in x_l..x_r {
                        buf[y * self.pixel_width + x] = fg;
                    }
                }
            }
        }

        // Now handle the characters
        for (row, chars) in frame.lines.iter().enumerate() {
            let (y_t, y_b) = self.y_bounds(row);
            for (col, (ch, pen)) in chars.iter().enumerate() {
                let (x_l, x_r) = self.x_bounds(col);
                if ch == &' ' {
                    continue;
                }
                let glyph = self.font_db.get_glyph_cache(
                    (*ch, Variant::from_pen(pen)),
                    self.font_size as f32,
                    &self.font_families,
                );
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
                    let y = y as usize;
                    for bmap_x in 0..metrics.width {
                        let x = x_offset + bmap_x as i32;
                        if x < 0 || x >= self.pixel_width as i32 {
                            continue;
                        }
                        let x = x as usize;

                        let mut pixel_row = row;
                        let mut pixel_col = col;
                        // Character reaches into previous row
                        if y < y_t {
                            if row == 0 {
                                continue;
                            } else {
                                pixel_row -= 1;
                            }
                        }
                        // Character reaches into next row
                        if y >= y_b {
                            if row >= frame.lines.len() - 1 {
                                continue;
                            } else {
                                pixel_row += 1;
                            }
                        }
                        // Character reaches into previous column
                        if x < x_l {
                            if col == 0 {
                                continue;
                            } else {
                                pixel_col -= 1;
                            }
                        }
                        // Character reaches into next column
                        if x >= x_r {
                            if col >= chars.len() - 1 {
                                continue;
                            } else {
                                pixel_col += 1;
                            }
                        }

                        let fg = text_attrs(
                            &frame.lines[pixel_row][pixel_col].1,
                            &frame.cursor,
                            pixel_col,
                            pixel_row,
                            &self.theme,
                        )
                        .foreground
                        .alpha(255);

                        let idx = y * self.pixel_width + x;
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

fn mix_colors(fg: RGBA8, bg: RGBA8, ratio: u8) -> RGBA8 {
    let ratio = ratio as u16;
    RGBA8::new(
        ((bg.r as u16) * (255 - ratio) / 255) as u8 + ((fg.r as u16) * ratio / 255) as u8,
        ((bg.g as u16) * (255 - ratio) / 255) as u8 + ((fg.g as u16) * ratio / 255) as u8,
        ((bg.b as u16) * (255 - ratio) / 255) as u8 + ((fg.b as u16) * ratio / 255) as u8,
        255,
    )
}
