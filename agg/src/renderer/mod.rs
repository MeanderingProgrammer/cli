use crate::fonts::CachingFontDb;
use crate::theme::Theme;
use crate::vt::Frame;
use avt::rgb::{RGB8, RGBA8};
use avt::{Color, Pen};
use clap::ValueEnum;
use imgref::ImgVec;

mod font;
mod svg;

#[derive(Debug, Clone, Default, ValueEnum)]
pub enum RendererName {
    #[default]
    Fontdue,
    // TODO: decide whether to remove this, unsure if it adds anything
    Resvg,
}

impl RendererName {
    pub fn get_renderer(&self, settings: Settings) -> Box<dyn Renderer> {
        match self {
            Self::Fontdue => Box::new(font::FontRenderer::new(settings)),
            Self::Resvg => Box::new(svg::SvgRenderer::new(settings)),
        }
    }
}

pub trait Renderer {
    fn render(&mut self, frame: Frame) -> ImgVec<RGBA8>;
    fn pixel_size(&self) -> (usize, usize);
}

#[derive(Debug)]
pub struct Settings {
    pub terminal_size: (usize, usize),
    pub font_db: CachingFontDb,
    pub font_families: Vec<String>,
    pub font_size: usize,
    pub line_height: f64,
    pub theme: Theme,
}

#[derive(Debug)]
struct TextAttrs {
    foreground: RGB8,
    background: Option<RGB8>,
}

fn text_attrs(
    pen: &Pen,
    cursor: &Option<(usize, usize)>,
    x: usize,
    y: usize,
    theme: &Theme,
) -> TextAttrs {
    let mut foreground = pen.foreground.clone();
    if pen.is_bold() {
        if let Some(Color::Indexed(n)) = foreground {
            if n < 8 {
                foreground = Some(Color::Indexed(n + 8));
            }
        }
    }

    let mut background = pen.background.clone();
    if pen.is_blink() {
        if let Some(Color::Indexed(n)) = background {
            if n < 8 {
                background = Some(Color::Indexed(n + 8));
            }
        }
    }

    let inverse = cursor.map_or(false, |(cx, cy)| cx == x && cy == y);
    if pen.is_inverse() ^ inverse {
        let fg = background.unwrap_or(Color::RGB(theme.background));
        let bg = foreground.unwrap_or(Color::RGB(theme.foreground));
        foreground = Some(fg);
        background = Some(bg);
    }

    TextAttrs {
        foreground: color_to_rgb(&foreground.unwrap_or(Color::RGB(theme.foreground)), theme),
        background: background.as_ref().map(|c| color_to_rgb(c, theme)),
    }
}

fn color_to_rgb(c: &Color, theme: &Theme) -> RGB8 {
    match c {
        Color::RGB(c) => *c,
        Color::Indexed(c) => theme.color(*c),
    }
}
