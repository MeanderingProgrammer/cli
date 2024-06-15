use crate::cell::Cell;
use crate::Pen;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Line {
    pub cells: Vec<Cell>,
    pub wrapped: bool,
}

impl Line {
    pub fn blank(cols: usize, pen: Pen) -> Self {
        Self {
            cells: vec![Cell::blank(pen); cols],
            wrapped: false,
        }
    }

    pub fn clear(&mut self, range: Range<usize>, pen: &Pen) {
        self.cells[range].fill(Cell::blank(pen.clone()));
    }

    pub fn print(&mut self, col: usize, cell: Cell) {
        self.cells[col] = cell;
    }

    pub fn insert(&mut self, col: usize, n: usize, cell: Cell) {
        self.cells[col..].rotate_right(n);
        self.cells[col..col + n].fill(cell);
    }

    pub fn delete(&mut self, col: usize, n: usize, pen: &Pen) {
        self.cells[col..].rotate_left(n);
        let start = self.cells.len() - n;
        self.cells[start..].fill(Cell::blank(pen.clone()));
    }

    pub fn extend(&mut self, mut other: Line, len: usize) -> (bool, Option<Line>) {
        let needed = len - self.len();
        if needed == 0 {
            return (true, Some(other));
        }
        if !self.wrapped {
            self.expand(len, &Pen::default());
            return (true, Some(other));
        }
        if !other.wrapped {
            other.trim();
        }
        if needed < other.len() {
            self.cells
                .extend(&mut other.cells[0..needed].iter().cloned());
            let mut cells = other.cells;
            cells.rotate_left(needed);
            cells.truncate(cells.len() - needed);
            (
                true,
                Some(Line {
                    cells,
                    wrapped: other.wrapped,
                }),
            )
        } else {
            self.cells.extend(&mut other.cells.into_iter());
            if !other.wrapped {
                self.wrapped = false;
                if self.len() < len {
                    self.expand(len, &Pen::default());
                }
                (true, None)
            } else {
                (false, None)
            }
        }
    }

    pub fn expand(&mut self, len: usize, pen: &Pen) {
        let tpl = Cell::blank(pen.clone());
        let filler = std::iter::repeat(tpl).take(len - self.len());
        self.cells.extend(filler);
    }

    pub fn contract(&mut self, len: usize) -> Option<Line> {
        if !self.wrapped {
            let trimmed_len = self.len() - self.trailers();
            self.cells.truncate(len.max(trimmed_len));
        }
        if self.len() > len {
            let mut rest = Line {
                cells: self.cells.split_off(len),
                wrapped: self.wrapped,
            };
            if !self.wrapped {
                rest.trim();
            }
            if rest.cells.is_empty() {
                None
            } else {
                self.wrapped = true;
                Some(rest)
            }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        self.cells.iter().cloned()
    }

    fn trim(&mut self) {
        let trailers = self.trailers();
        if trailers > 0 {
            self.cells.truncate(self.len() - trailers);
        }
    }

    fn trailers(&self) -> usize {
        self.cells
            .iter()
            .rev()
            .take_while(|cell| cell.is_default())
            .count()
    }
}
