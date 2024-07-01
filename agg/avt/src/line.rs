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

    pub fn clear(&mut self, range: Range<usize>, pen: &Pen) {
        self.cells[range].fill(Cell::blank(pen.clone()));
    }
}

#[cfg(test)]
mod tests {
    use crate::{line::Line, Cell, Pen};

    #[test]
    fn print() {
        let mut line = Line::blank(4, Pen::default());
        line.print(1, 'a'.into());
        line.print(2, 'b'.into());
        line.print(3, 'c'.into());

        let expected: Vec<Cell> = vec![' '.into(), 'a'.into(), 'b'.into(), 'c'.into()];

        assert_eq!(expected, line.cells);
    }

    #[test]
    fn insert() {
        let mut line = Line::blank(4, Pen::default());
        line.insert(1, 2, 'a'.into());

        let expected: Vec<Cell> = vec![' '.into(), 'a'.into(), 'a'.into(), ' '.into()];

        assert_eq!(expected, line.cells);
    }

    #[test]
    fn delete() {
        let mut line = Line::blank(4, Pen::default());
        line.print(0, 'a'.into());
        line.print(1, 'b'.into());
        line.print(2, 'c'.into());
        line.print(3, 'd'.into());
        line.delete(1, 2, &Pen::default());

        let expected: Vec<Cell> = vec!['a'.into(), 'd'.into(), ' '.into(), ' '.into()];

        assert_eq!(expected, line.cells);
    }
}
