use crate::pen::Pen;

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub pen: Pen,
}

impl From<char> for Cell {
    fn from(value: char) -> Self {
        Self::new(value, Pen::default())
    }
}

impl Cell {
    pub fn new(ch: char, pen: Pen) -> Self {
        Self { ch, pen }
    }

    pub fn blank(pen: Pen) -> Self {
        Self::new(' ', pen)
    }
}
