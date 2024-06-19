use rgb::RGB8;

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Indexed(u8),
    RGB(RGB8),
}

impl From<u16> for Color {
    fn from(value: u16) -> Self {
        Self::Indexed(value as u8)
    }
}

impl From<(u16, u16, u16)> for Color {
    fn from((r, g, b): (u16, u16, u16)) -> Self {
        Self::RGB(RGB8::new(r as u8, g as u8, b as u8))
    }
}
