use crate::pen::Pen;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone)]
pub struct Cell(pub char, pub Pen);

impl Cell {
    pub fn blank(pen: Pen) -> Self {
        Self(' ', pen)
    }

    pub fn is_default(&self) -> bool {
        self.0 == ' ' && self.1.is_default()
    }

    pub fn char_width(&self) -> usize {
        UnicodeWidthChar::width(self.0).unwrap_or(0)
    }
}
