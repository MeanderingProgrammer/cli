use crate::cell::Cell;
use crate::line::Line;
use crate::Pen;
use std::cmp::Ordering;
use std::ops::Range;

#[derive(Debug)]
pub struct Buffer {
    pub cols: usize,
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
    pub fn new(cols: usize, rows: usize, pen: Option<&Pen>) -> Self {
        let default_pen = Pen::default();
        let pen = pen.unwrap_or(&default_pen);
        let lines = vec![Line::blank(cols, pen.clone()); rows];
        Self { cols, rows, lines }
    }

    pub fn print(&mut self, (col, row): (usize, usize), cell: Cell) {
        self.row_mut(row).print(col, cell);
    }

    pub fn wrap(&mut self, row: usize) {
        self.row_mut(row).wrapped = true;
    }

    pub fn insert(&mut self, (col, row): (usize, usize), mut n: usize, cell: Cell) {
        n = n.min(self.cols - col);
        self.row_mut(row).insert(col, n, cell);
    }

    pub fn delete(&mut self, (col, row): (usize, usize), mut n: usize, pen: &Pen) {
        n = n.min(self.cols - col);
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
            EraseMode::WholeView => {
                self.clear(0..self.rows, pen);
            }
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

    pub fn scroll_up(&mut self, range: Range<usize>, mut n: usize, pen: &Pen) {
        n = n.min(range.end - range.start);
        if range.end - 1 < self.rows - 1 {
            self.row_mut(range.end - 1).wrapped = false;
        }
        if range.start == 0 {
            if range.end == self.rows {
                self.extend(n, self.cols);
            } else {
                let line = Line::blank(self.cols, pen.clone());
                let index = self.lines.len() - self.rows + range.end;
                for _ in 0..n {
                    self.lines.insert(index, line.clone());
                }
            }
        } else {
            self.row_mut(range.start - 1).wrapped = false;
            let end = range.end;
            self.view_mut()[range].rotate_left(n);
            self.clear((end - n)..end, pen);
        }
    }

    pub fn scroll_down(&mut self, range: Range<usize>, mut n: usize, pen: &Pen) {
        let (start, end) = (range.start, range.end);
        n = n.min(end - start);
        self.view_mut()[range].rotate_right(n);
        self.clear(start..start + n, pen);
        if start > 0 {
            self.row_mut(start - 1).wrapped = false;
        }
        self.row_mut(end - 1).wrapped = false;
    }

    pub fn resize(
        &mut self,
        new_cols: usize,
        new_rows: usize,
        mut cursor: (usize, usize),
    ) -> (usize, usize) {
        let old_cols = self.cols;
        let mut old_rows = self.rows;
        let cursor_log_pos = self.logical_position(cursor, old_cols, old_rows);
        if new_cols != old_cols {
            self.lines = reflow(self.lines.drain(..), new_cols);
            let line_count = self.lines.len();
            if line_count < old_rows {
                self.extend(old_rows - line_count, new_cols);
            }
            let cursor_rel_pos = self.relative_position(cursor_log_pos, new_cols, old_rows);
            cursor.0 = cursor_rel_pos.0;
            if cursor_rel_pos.1 >= 0 {
                cursor.1 = cursor_rel_pos.1 as usize;
            } else {
                cursor.1 = 0;
                old_rows += (-cursor_rel_pos.1) as usize;
            }
        }

        let line_count = self.lines.len();
        match new_rows.cmp(&old_rows) {
            Ordering::Less => {
                let height_delta = old_rows - new_rows;
                let inverted_cursor_row = old_rows - 1 - cursor.1;
                let excess = height_delta.min(inverted_cursor_row);
                if excess > 0 {
                    self.lines.truncate(line_count - excess);
                    self.lines.last_mut().unwrap().wrapped = false;
                }
                cursor.1 -= height_delta - excess;
            }
            Ordering::Greater => {
                let mut height_delta = new_rows - old_rows;
                let scrollback_size = line_count - old_rows.min(line_count);
                let cursor_row_shift = scrollback_size.min(height_delta);
                height_delta -= cursor_row_shift;
                if cursor.1 < old_rows {
                    cursor.1 += cursor_row_shift;
                }
                if height_delta > 0 {
                    self.extend(height_delta, new_cols);
                }
            }
            Ordering::Equal => (),
        }
        self.cols = new_cols;
        self.rows = new_rows;
        cursor
    }

    fn logical_position(&self, pos: (usize, usize), cols: usize, rows: usize) -> (usize, usize) {
        let vis_row_offset = self.lines.len() - rows;
        let mut log_col_offset = 0;
        let abs_row = pos.1 + vis_row_offset;
        let last_available_row = abs_row.min(self.lines.len());
        let mut log_row = abs_row - last_available_row;
        for line in self.lines.iter().take(abs_row) {
            if line.wrapped {
                log_col_offset += cols;
            } else {
                log_col_offset = 0;
                log_row += 1;
            }
        }
        (pos.0 + log_col_offset, log_row)
    }

    fn relative_position(&self, pos: (usize, usize), cols: usize, rows: usize) -> (usize, isize) {
        let mut rel_col = pos.0;
        let mut rel_row = 0;
        let mut r = 0;
        let last_row = self.lines.len() - 1;
        while r < pos.1 && rel_row < last_row {
            if !self.lines[rel_row].wrapped {
                r += 1;
            }
            rel_row += 1;
        }
        while rel_col >= cols && self.lines[rel_row].wrapped {
            rel_col -= cols;
            rel_row += 1;
        }
        rel_col = rel_col.min(cols - 1);
        let rel_row_offset = self.lines.len() - rows;
        (rel_col, (rel_row as isize - rel_row_offset as isize))
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

struct Reflow<I>
where
    I: Iterator<Item = Line>,
{
    pub iter: I,
    pub cols: usize,
    pub rest: Option<Line>,
}

pub fn reflow<I: Iterator<Item = Line>>(iter: I, cols: usize) -> Vec<Line> {
    let lines: Vec<Line> = Reflow {
        iter,
        cols,
        rest: None,
    }
    .collect();
    assert!(lines.iter().all(|l| l.len() == cols));
    lines
}

impl<I: Iterator<Item = Line>> Iterator for Reflow<I> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut line) = self.rest.take().or_else(|| self.iter.next()) {
            match self.cols.cmp(&line.len()) {
                Ordering::Less => {
                    self.rest = line.contract(self.cols);
                    return Some(line);
                }
                Ordering::Equal => return Some(line),
                Ordering::Greater => match self.iter.next() {
                    Some(next_line) => match line.extend(next_line, self.cols) {
                        (true, Some(rest)) => {
                            self.rest = Some(rest);
                            return Some(line);
                        }
                        (true, None) => {
                            return Some(line);
                        }
                        (false, _) => {
                            self.rest = Some(line);
                        }
                    },
                    None => {
                        line.expand(self.cols, &Pen::default());
                        line.wrapped = false;
                        return Some(line);
                    }
                },
            }
        }

        self.rest.take().map(|mut line| {
            line.expand(self.cols, &Pen::default());
            line.wrapped = false;
            line
        })
    }
}
