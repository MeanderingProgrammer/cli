use rgb::RGB8;

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Indexed(u8),
    RGB(RGB8),
}
