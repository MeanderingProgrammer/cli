use crate::cell::Cell;
use crate::Pen;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Line {
    pub cells: Vec<Cell>,
    pub wrapped: bool,
}

impl Line {
    fn new(cells: Vec<Cell>) -> Self {
        Self {
            cells,
            wrapped: false,
        }
    }

    pub fn blank(cols: usize, pen: &Pen) -> Self {
        Self::new(vec![Cell::blank(pen); cols])
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
        self.cells[start..].fill(Cell::blank(pen));
    }

    pub fn clear(&mut self, range: Range<usize>, pen: &Pen) {
        self.cells[range].fill(Cell::blank(pen));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pen;

    #[test]
    fn print() {
        let mut line = Line::blank(4, &Pen::default());
        line.print(1, 'a'.into());
        line.print(2, 'b'.into());
        line.print(3, 'c'.into());

        assert_eq!(
            line.cells,
            vec![' '.into(), 'a'.into(), 'b'.into(), 'c'.into()]
        );
    }

    #[test]
    fn insert() {
        let mut line = Line::blank(4, &Pen::default());
        line.insert(1, 2, 'a'.into());

        assert_eq!(
            line.cells,
            vec![' '.into(), 'a'.into(), 'a'.into(), ' '.into()]
        );
    }

    #[test]
    fn delete() {
        let mut line = Line::new(vec!['a'.into(), 'b'.into(), 'c'.into(), 'd'.into()]);
        line.delete(1, 2, &Pen::default());

        assert_eq!(
            line.cells,
            vec!['a'.into(), 'd'.into(), ' '.into(), ' '.into()]
        );
    }

    #[test]
    fn clear() {
        let mut line = Line::new(vec!['a'.into(), 'b'.into(), 'c'.into(), 'd'.into()]);
        line.clear(1..3, &Pen::default());

        assert_eq!(
            line.cells,
            vec!['a'.into(), ' '.into(), ' '.into(), 'd'.into()]
        );
    }
}
