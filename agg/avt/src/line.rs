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

    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        self.cells.iter().cloned()
    }
}
