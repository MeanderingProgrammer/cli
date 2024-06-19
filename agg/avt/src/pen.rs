use crate::Color;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pen {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub intensity: Intensity,
    pub attrs: u8,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Intensity {
    #[default]
    Normal,
    Bold,
    Faint,
}

const ITALIC_MASK: u8 = 1;
const UNDERLINE_MASK: u8 = 1 << 1;
const STRIKETHROUGH_MASK: u8 = 1 << 2;
const BLINK_MASK: u8 = 1 << 3;
const INVERSE_MASK: u8 = 1 << 4;

impl Pen {
    pub fn is_bold(&self) -> bool {
        self.intensity == Intensity::Bold
    }

    pub fn is_faint(&self) -> bool {
        self.intensity == Intensity::Faint
    }

    pub fn is_italic(&self) -> bool {
        (self.attrs & ITALIC_MASK) != 0
    }

    pub fn is_underline(&self) -> bool {
        (self.attrs & UNDERLINE_MASK) != 0
    }

    pub fn is_strikethrough(&self) -> bool {
        (self.attrs & STRIKETHROUGH_MASK) != 0
    }

    pub fn is_blink(&self) -> bool {
        (self.attrs & BLINK_MASK) != 0
    }

    pub fn is_inverse(&self) -> bool {
        (self.attrs & INVERSE_MASK) != 0
    }

    pub fn set_italic(&mut self) {
        self.attrs |= ITALIC_MASK;
    }

    pub fn set_underline(&mut self) {
        self.attrs |= UNDERLINE_MASK;
    }

    pub fn set_blink(&mut self) {
        self.attrs |= BLINK_MASK;
    }

    pub fn set_strikethrough(&mut self) {
        self.attrs |= STRIKETHROUGH_MASK;
    }

    pub fn set_inverse(&mut self) {
        self.attrs |= INVERSE_MASK;
    }

    pub fn unset_italic(&mut self) {
        self.attrs &= !ITALIC_MASK;
    }

    pub fn unset_underline(&mut self) {
        self.attrs &= !UNDERLINE_MASK;
    }

    pub fn unset_blink(&mut self) {
        self.attrs &= !BLINK_MASK;
    }

    pub fn unset_strikethrough(&mut self) {
        self.attrs &= !STRIKETHROUGH_MASK;
    }

    pub fn unset_inverse(&mut self) {
        self.attrs &= !INVERSE_MASK;
    }

    pub fn is_default(&self) -> bool {
        self.foreground.is_none()
            && self.background.is_none()
            && self.intensity == Intensity::default()
            && !self.is_italic()
            && !self.is_underline()
            && !self.is_strikethrough()
            && !self.is_blink()
            && !self.is_inverse()
    }
}
