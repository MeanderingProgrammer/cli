use crate::renderer::{color_to_rgb, text_attrs, Renderer, Settings};
use crate::theme::Theme;
use crate::vt::Frame;
use avt::{Color, Pen};
use imgref::ImgVec;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use rgb::{FromSlice, RGBA8};
use std::fmt::Write as _;
use std::sync::Arc;

#[derive(Debug)]
pub struct SvgRenderer<'a> {
    terminal_size: (usize, usize),
    theme: Theme,
    pixel_width: usize,
    pixel_height: usize,
    char_width: f64,
    row_height: f64,
    options: Options<'a>,
    transform: Transform,
    header: String,
}

fn color_to_style(color: &Color, theme: &Theme) -> String {
    let c = color_to_rgb(color, theme);
    format!("fill: rgb({},{},{})", c.r, c.g, c.b)
}

fn text_class(pen: &Pen) -> String {
    let mut class = "".to_owned();
    if pen.is_bold() {
        class.push_str("br");
    }
    if pen.is_italic() {
        class.push_str(" it");
    }
    if pen.is_underline() {
        class.push_str(" un");
    }
    class
}

fn text_style(foreground: Option<Color>, theme: &Theme) -> String {
    foreground
        .map(|c| color_to_style(&c, theme))
        .unwrap_or_else(|| "".to_owned())
}

fn rect_style(background: Option<Color>, theme: &Theme) -> String {
    background
        .map(|c| color_to_style(&c, theme))
        .unwrap_or_else(|| "".to_owned())
}

impl<'a> SvgRenderer<'a> {
    pub fn new(settings: Settings) -> Self {
        let char_width = 100.0 / (settings.terminal_size.0 as f64 + 2.0);
        let font_size = settings.font_size as f64;
        let row_height = font_size * settings.line_height;
        let options = Options {
            fontdb: Arc::new(settings.font_db),
            ..Options::default()
        };
        let transform = Transform::default();

        let header = Self::header(
            settings.terminal_size,
            settings.font_families.join(","),
            font_size,
            row_height,
            &settings.theme,
        );

        let mut svg = header.clone();
        svg.push_str(Self::footer());
        let tree = Tree::from_str(&svg, &options).unwrap();
        let screen_size = tree.size();
        let pixel_width = screen_size.width() as usize;
        let pixel_height = screen_size.height() as usize;

        Self {
            terminal_size: settings.terminal_size,
            theme: settings.theme,
            pixel_width,
            pixel_height,
            char_width,
            row_height,
            options,
            transform,
            header,
        }
    }

    fn header(
        (cols, rows): (usize, usize),
        font_family: String,
        font_size: f64,
        row_height: f64,
        theme: &Theme,
    ) -> String {
        let width = (cols + 2) as f64 * (font_size * 0.6);
        let height = (rows + 1) as f64 * row_height;
        let x = 1.0 * 100.0 / (cols as f64 + 2.0);
        let y = 0.5 * 100.0 / (rows as f64 + 1.0);

        format!(
            r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{}" height="{}" font-size="{}px" font-family="{}">
<style>
.br {{ font-weight: bold }}
.it {{ font-style: italic }}
.un {{ text-decoration: underline }}
</style>
<rect width="100%" height="100%" rx="{}" ry="{}" style="fill: {}" />
<svg x="{:.3}%" y="{:.3}%" style="fill: {}">"#,
            width, height, font_size, font_family, 4, 4, theme.background, x, y, theme.foreground
        )
    }

    fn footer() -> &'static str {
        "</svg></svg>"
    }

    fn push_lines(&self, svg: &mut String, frame: Frame) {
        self.push_background(svg, &frame);
        self.push_text(svg, &frame);
    }

    fn push_background(&self, svg: &mut String, frame: &Frame) {
        let (cols, rows) = self.terminal_size;

        svg.push_str(r#"<g style="shape-rendering: optimizeSpeed">"#);

        for (row, line) in frame.lines.iter().enumerate() {
            let y = 100.0 * (row as f64) / (rows as f64 + 1.0);

            for (col, (_ch, pen)) in line.iter().enumerate() {
                let attrs = text_attrs(pen, &frame.cursor, col, row, &self.theme);

                if attrs.background.is_none() {
                    continue;
                }

                let x = 100.0 * (col as f64) / (cols as f64 + 2.0);
                let style = rect_style(attrs.background, &self.theme);

                let _ = write!(
                    svg,
                    r#"<rect x="{:.3}%" y="{:.3}%" width="{:.3}%" height="{:.3}" style="{}" />"#,
                    x, y, self.char_width, self.row_height, style
                );
            }
        }

        svg.push_str("</g>");
    }

    fn push_text(&self, svg: &mut String, frame: &Frame) {
        let (cols, rows) = self.terminal_size;

        svg.push_str(r#"<text class="default-text-fill">"#);

        for (row, line) in frame.lines.iter().enumerate() {
            let y = 100.0 * (row as f64) / (rows as f64 + 1.0);
            let mut did_dy = false;

            let _ = write!(svg, r#"<tspan y="{y:.3}%">"#);

            for (col, (ch, pen)) in line.iter().enumerate() {
                if ch == &' ' {
                    continue;
                }

                let attrs = text_attrs(pen, &frame.cursor, col, row, &self.theme);

                svg.push_str("<tspan ");

                if !did_dy {
                    svg.push_str(r#"dy="1em" "#);
                    did_dy = true;
                }

                let x = 100.0 * (col as f64) / (cols as f64 + 2.0);
                let class = text_class(pen);
                let style = text_style(attrs.foreground, &self.theme);

                let _ = write!(svg, r#"x="{x:.3}%" class="{class}" style="{style}">"#);

                match ch {
                    '\'' => {
                        svg.push_str("&#39;");
                    }

                    '"' => {
                        svg.push_str("&quot;");
                    }

                    '&' => {
                        svg.push_str("&amp;");
                    }

                    '>' => {
                        svg.push_str("&gt;");
                    }

                    '<' => {
                        svg.push_str("&lt;");
                    }

                    _ => {
                        svg.push(*ch);
                    }
                }

                svg.push_str("</tspan>");
            }

            svg.push_str("</tspan>");
        }

        svg.push_str("</text>");
    }
}

impl<'a> Renderer for SvgRenderer<'a> {
    fn render(&mut self, frame: Frame) -> ImgVec<RGBA8> {
        let mut svg = self.header.clone();
        self.push_lines(&mut svg, frame);
        svg.push_str(Self::footer());
        let tree = Tree::from_str(&svg, &self.options).unwrap();

        let mut pixmap = Pixmap::new(self.pixel_width as u32, self.pixel_height as u32).unwrap();

        ::resvg::render(&tree, self.transform, &mut pixmap.as_mut());
        let buf = pixmap.take().as_rgba().to_vec();

        ImgVec::new(buf, self.pixel_width, self.pixel_height)
    }

    fn pixel_size(&self) -> (usize, usize) {
        (self.pixel_width, self.pixel_height)
    }
}
