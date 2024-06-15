#[derive(Debug)]
pub enum Charset {
    Ascii,
    Drawing,
}

const SPECIAL_GFX_CHARS: [char; 31] = [
    '♦', '▒', '␉', '␌', '␍', '␊', '°', '±', '␤', '␋', '┘', '┐', '┌', '└', '┼', '⎺', '⎻', '─', '⎼',
    '⎽', '├', '┤', '┴', '┬', '│', '≤', '≥', 'π', '≠', '£', '⋅',
];

impl Charset {
    pub fn translate(&self, input: char) -> char {
        match self {
            Charset::Ascii => input,
            Charset::Drawing => {
                if ('\x60'..'\x7f').contains(&input) {
                    SPECIAL_GFX_CHARS[(input as usize) - 0x60]
                } else {
                    input
                }
            }
        }
    }
}
