#[derive(Debug, Clone)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub visible: bool,
}

impl Cursor {
    pub fn position(&self) -> (usize, usize) {
        (self.col, self.row)
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            col: 0,
            row: 0,
            visible: true,
        }
    }
}

impl From<Cursor> for Option<(usize, usize)> {
    fn from(cursor: Cursor) -> Self {
        if cursor.visible {
            Some((cursor.col, cursor.row))
        } else {
            None
        }
    }
}
