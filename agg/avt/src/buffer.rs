use crate::cell::Cell;
use crate::line::Line;
use crate::Pen;
use std::ops::Range;

#[derive(Debug)]
pub struct Buffer {
    cols: usize,
    rows: usize,
    lines: Vec<Line>,
}

pub enum EraseMode {
    NextChars(usize),
    FromCursorToEndOfView,
    FromStartOfViewToCursor,
    WholeView,
    FromCursorToEndOfLine,
    FromStartOfLineToCursor,
    WholeLine,
}

impl Buffer {
    pub fn new(cols: usize, rows: usize, pen: Pen) -> Self {
        let lines = vec![Line::blank(cols, pen); rows];
        Self { cols, rows, lines }
    }

    pub fn print(&mut self, (col, row): (usize, usize), cell: Cell) {
        self.row_mut(row).print(col, cell);
    }

    pub fn wrap(&mut self, row: usize) {
        self.row_mut(row).wrapped = true;
    }

    pub fn insert(&mut self, (col, row): (usize, usize), n: usize, cell: Cell) {
        let n = n.min(self.cols - col);
        self.row_mut(row).insert(col, n, cell);
    }

    pub fn delete(&mut self, (col, row): (usize, usize), n: usize, pen: &Pen) {
        let n = n.min(self.cols - col);
        let line = &mut self.row_mut(row);
        line.delete(col, n, pen);
        line.wrapped = false;
    }

    pub fn erase(&mut self, (col, row): (usize, usize), mode: EraseMode, pen: &Pen) {
        match mode {
            EraseMode::NextChars(mut n) => {
                n = n.min(self.cols - col);
                let end = col + n;
                let clear_wrap = end == self.cols;
                let line = &mut self.row_mut(row);
                line.clear(col..end, pen);
                if clear_wrap {
                    line.wrapped = false;
                }
            }
            EraseMode::FromCursorToEndOfView => {
                let range = col..self.cols;
                let line = &mut self.row_mut(row);
                line.wrapped = false;
                line.clear(range, pen);
                self.clear((row + 1)..self.rows, pen);
            }
            EraseMode::FromStartOfViewToCursor => {
                let range = 0..(col + 1).min(self.cols);
                self.row_mut(row).clear(range, pen);
                self.clear(0..row, pen);
            }
            EraseMode::WholeView => self.clear(0..self.rows, pen),
            EraseMode::FromCursorToEndOfLine => {
                let range = col..self.cols;
                let line = &mut self.row_mut(row);
                line.clear(range, pen);
                line.wrapped = false;
            }
            EraseMode::FromStartOfLineToCursor => {
                let range = 0..(col + 1).min(self.cols);
                self.row_mut(row).clear(range, pen);
            }
            EraseMode::WholeLine => {
                let range = 0..self.cols;
                let line = &mut self.row_mut(row);
                line.clear(range, pen);
                line.wrapped = false;
            }
        }
    }

    pub fn scroll_up(&mut self, range: Range<usize>, n: usize, pen: &Pen) {
        let (start, end) = (range.start, range.end);
        let n = n.min(end - start);
        if end - 1 < self.rows - 1 {
            self.row_mut(end - 1).wrapped = false;
        }
        if start == 0 {
            if end == self.rows {
                self.extend(n, self.cols);
            } else {
                let line = Line::blank(self.cols, pen.clone());
                let index = self.lines.len() - self.rows + end;
                for _ in 0..n {
                    self.lines.insert(index, line.clone());
                }
            }
        } else {
            self.row_mut(start - 1).wrapped = false;
            self.view_mut()[range].rotate_left(n);
            self.clear((end - n)..end, pen);
        }
    }

    pub fn scroll_down(&mut self, range: Range<usize>, n: usize, pen: &Pen) {
        let (start, end) = (range.start, range.end);
        let n = n.min(end - start);
        self.view_mut()[range].rotate_right(n);
        self.clear(start..start + n, pen);
        if start > 0 {
            self.row_mut(start - 1).wrapped = false;
        }
        self.row_mut(end - 1).wrapped = false;
    }

    pub fn view(&self) -> &[Line] {
        &self.lines[self.lines.len() - self.rows..]
    }

    fn view_mut(&mut self) -> &mut [Line] {
        let len = self.lines.len();
        &mut self.lines[len - self.rows..]
    }

    fn row_mut(&mut self, row: usize) -> &mut Line {
        &mut self.view_mut()[row]
    }

    fn clear(&mut self, range: Range<usize>, pen: &Pen) {
        let line = Line::blank(self.cols, pen.clone());
        self.view_mut()[range].fill(line);
    }

    fn extend(&mut self, n: usize, cols: usize) {
        let line = Line::blank(cols, Pen::default());
        let filler = std::iter::repeat(line).take(n);
        self.lines.extend(filler);
    }
}
