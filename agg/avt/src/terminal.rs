use crate::buffer::{Buffer, EraseMode};
use crate::cell::Cell;
use crate::charset::Charset;
use crate::cursor::Cursor;
use crate::dirty_lines::DirtyLines;
use crate::line::Line;
use crate::parser::Params;
use crate::pen::Intensity;
use crate::saved_ctx::SavedCtx;
use crate::tabs::Tabs;
use crate::{Color, Pen};
use rgb::RGB8;

#[derive(Debug)]
enum BufferType {
    Primary,
    Alternate,
}

#[derive(Debug)]
pub struct Terminal {
    cols: usize,
    rows: usize,
    buffer: Buffer,
    other_buffer: Buffer,
    active_buffer_type: BufferType,
    cursor: Cursor,
    pen: Pen,
    charsets: [Charset; 2],
    active_charset: usize,
    tabs: Tabs,
    insert_mode: bool,
    origin_mode: bool,
    auto_wrap_mode: bool,
    new_line_mode: bool,
    next_print_wraps: bool,
    top_margin: usize,
    bottom_margin: usize,
    saved_ctx: SavedCtx,
    alternate_saved_ctx: SavedCtx,
    dirty_lines: DirtyLines,
}

impl Terminal {
    pub fn new((cols, rows): (usize, usize)) -> Self {
        Self {
            cols,
            rows,
            buffer: Buffer::new(cols, rows, None),
            other_buffer: Buffer::new(cols, rows, None),
            active_buffer_type: BufferType::Primary,
            tabs: Tabs::new(cols),
            cursor: Cursor::default(),
            pen: Pen::default(),
            charsets: [Charset::Ascii, Charset::Ascii],
            active_charset: 0,
            insert_mode: false,
            origin_mode: false,
            auto_wrap_mode: true,
            new_line_mode: false,
            next_print_wraps: false,
            top_margin: 0,
            bottom_margin: (rows - 1),
            saved_ctx: SavedCtx::default(),
            alternate_saved_ctx: SavedCtx::default(),
            dirty_lines: DirtyLines::new(rows),
        }
    }

    pub fn changes(&mut self) -> Vec<usize> {
        let changes = self.dirty_lines.to_vec();
        self.dirty_lines.clear();
        changes
    }

    pub fn cursor(&self) -> Option<(usize, usize)> {
        self.cursor.clone().into()
    }

    pub fn view(&self) -> &[Line] {
        self.buffer.view()
    }

    fn save_cursor(&mut self) {
        self.saved_ctx.cursor_col = self.cursor.col.min(self.cols - 1);
        self.saved_ctx.cursor_row = self.cursor.row;
        self.saved_ctx.pen = self.pen.clone();
        self.saved_ctx.origin_mode = self.origin_mode;
        self.saved_ctx.auto_wrap_mode = self.auto_wrap_mode;
    }

    fn restore_cursor(&mut self) {
        self.cursor.col = self.saved_ctx.cursor_col;
        self.cursor.row = self.saved_ctx.cursor_row;
        self.pen = self.saved_ctx.pen.clone();
        self.origin_mode = self.saved_ctx.origin_mode;
        self.auto_wrap_mode = self.saved_ctx.auto_wrap_mode;
        self.next_print_wraps = false;
    }

    fn move_cursor_to_col(&mut self, col: usize) {
        if col >= self.cols {
            self.do_move_cursor_to_col(self.cols - 1);
        } else {
            self.do_move_cursor_to_col(col);
        }
    }

    fn do_move_cursor_to_col(&mut self, col: usize) {
        self.cursor.col = col;
        self.next_print_wraps = false;
    }

    fn move_cursor_to_row(&mut self, mut row: usize) {
        let top = self.actual_top_margin();
        let bottom = self.actual_bottom_margin();
        row = (top + row).max(top).min(bottom);
        self.do_move_cursor_to_row(row);
    }

    fn do_move_cursor_to_row(&mut self, row: usize) {
        self.cursor.col = self.cursor.col.min(self.cols - 1);
        self.cursor.row = row;
        self.next_print_wraps = false;
    }

    fn move_cursor_to_rel_col(&mut self, rel_col: isize) {
        let new_col = self.cursor.col as isize + rel_col;
        if new_col < 0 {
            self.do_move_cursor_to_col(0);
        } else if new_col as usize >= self.cols {
            self.do_move_cursor_to_col(self.cols - 1);
        } else {
            self.do_move_cursor_to_col(new_col as usize);
        }
    }

    fn move_cursor_home(&mut self) {
        self.do_move_cursor_to_col(0);
        self.do_move_cursor_to_row(self.actual_top_margin());
    }

    fn move_cursor_to_next_tab(&mut self, n: usize) {
        let next_tab = self.tabs.after(self.cursor.col, n).unwrap_or(self.cols - 1);
        self.move_cursor_to_col(next_tab);
    }

    fn move_cursor_to_prev_tab(&mut self, n: usize) {
        let prev_tab = self.tabs.before(self.cursor.col, n).unwrap_or(0);
        self.move_cursor_to_col(prev_tab);
    }

    fn move_cursor_down_with_scroll(&mut self) {
        if self.cursor.row == self.bottom_margin {
            self.scroll_up_in_region(1);
        } else if self.cursor.row < self.rows - 1 {
            self.do_move_cursor_to_row(self.cursor.row + 1);
        }
    }

    fn cursor_down(&mut self, n: usize) {
        let new_y = if self.cursor.row > self.bottom_margin {
            (self.rows - 1).min(self.cursor.row + n)
        } else {
            self.bottom_margin.min(self.cursor.row + n)
        };
        self.do_move_cursor_to_row(new_y);
    }

    fn cursor_up(&mut self, n: usize) {
        let mut new_y = (self.cursor.row as isize) - (n as isize);
        new_y = if self.cursor.row < self.top_margin {
            new_y.max(0)
        } else {
            new_y.max(self.top_margin as isize)
        };
        self.do_move_cursor_to_row(new_y as usize);
    }

    fn actual_top_margin(&self) -> usize {
        if self.origin_mode {
            self.top_margin
        } else {
            0
        }
    }

    fn actual_bottom_margin(&self) -> usize {
        if self.origin_mode {
            self.bottom_margin
        } else {
            self.rows - 1
        }
    }

    fn scroll_up_in_region(&mut self, n: usize) {
        let range = self.top_margin..self.bottom_margin + 1;
        self.buffer.scroll_up(range.clone(), n, &self.pen);
        self.dirty_lines.extend(range);
    }

    fn scroll_down_in_region(&mut self, n: usize) {
        let range = self.top_margin..self.bottom_margin + 1;
        self.buffer.scroll_down(range.clone(), n, &self.pen);
        self.dirty_lines.extend(range);
    }

    fn set_tab(&mut self) {
        if 0 < self.cursor.col && self.cursor.col < self.cols {
            self.tabs.set(self.cursor.col);
        }
    }

    fn clear_tab(&mut self) {
        self.tabs.unset(self.cursor.col);
    }

    fn clear_all_tabs(&mut self) {
        self.tabs.clear();
    }

    fn switch_to_alternate_buffer(&mut self) {
        if let BufferType::Primary = self.active_buffer_type {
            self.active_buffer_type = BufferType::Alternate;
            std::mem::swap(&mut self.saved_ctx, &mut self.alternate_saved_ctx);
            std::mem::swap(&mut self.buffer, &mut self.other_buffer);
            self.buffer = Buffer::new(self.cols, self.rows, Some(&self.pen));
            self.dirty_lines.extend(0..self.rows);
        }
    }

    fn switch_to_primary_buffer(&mut self) {
        if let BufferType::Alternate = self.active_buffer_type {
            self.active_buffer_type = BufferType::Primary;
            std::mem::swap(&mut self.saved_ctx, &mut self.alternate_saved_ctx);
            std::mem::swap(&mut self.buffer, &mut self.other_buffer);
            self.dirty_lines.extend(0..self.rows);
        }
    }

    fn reflow(&mut self) {
        if self.cols != self.buffer.cols {
            self.next_print_wraps = false;
        }
        (self.cursor.col, self.cursor.row) =
            self.buffer
                .resize(self.cols, self.rows, self.cursor.position());
        self.dirty_lines.resize(self.rows);
        self.dirty_lines.extend(0..self.rows);
        if self.saved_ctx.cursor_col >= self.cols {
            self.saved_ctx.cursor_col = self.cols - 1;
        }
        if self.saved_ctx.cursor_row >= self.rows {
            self.saved_ctx.cursor_row = self.rows - 1;
        }
    }

    fn soft_reset(&mut self) {
        self.cursor.visible = true;
        self.top_margin = 0;
        self.bottom_margin = self.rows - 1;
        self.insert_mode = false;
        self.origin_mode = false;
        self.pen = Pen::default();
        self.charsets = [Charset::Ascii, Charset::Ascii];
        self.active_charset = 0;
        self.saved_ctx = SavedCtx::default();
    }

    fn hard_reset(&mut self) {
        self.buffer = Buffer::new(self.cols, self.rows, None);
        self.other_buffer = Buffer::new(self.cols, self.rows, None);
        self.active_buffer_type = BufferType::Primary;
        self.tabs = Tabs::new(self.cols);
        self.cursor = Cursor::default();
        self.pen = Pen::default();
        self.charsets = [Charset::Ascii, Charset::Ascii];
        self.active_charset = 0;
        self.insert_mode = false;
        self.origin_mode = false;
        self.auto_wrap_mode = true;
        self.new_line_mode = false;
        self.next_print_wraps = false;
        self.top_margin = 0;
        self.bottom_margin = self.rows - 1;
        self.saved_ctx = SavedCtx::default();
        self.alternate_saved_ctx = SavedCtx::default();
        self.dirty_lines = DirtyLines::new(self.rows);
    }
}

// https://en.wikipedia.org/wiki/ANSI_escape_code
impl Terminal {
    pub fn print(&mut self, mut input: char) {
        input = self.charsets[self.active_charset].map(input);
        let cell = Cell(input, self.pen.clone());
        if self.auto_wrap_mode && self.next_print_wraps {
            self.do_move_cursor_to_col(0);
            if self.cursor.row == self.bottom_margin {
                self.buffer.wrap(self.cursor.row);
                self.scroll_up_in_region(1);
            } else if self.cursor.row < self.rows - 1 {
                self.buffer.wrap(self.cursor.row);
                self.do_move_cursor_to_row(self.cursor.row + 1);
            }
        }
        let next_col = self.cursor.col + 1;
        if next_col >= self.cols {
            self.buffer.print((self.cols - 1, self.cursor.row), cell);
            if self.auto_wrap_mode {
                self.do_move_cursor_to_col(self.cols);
                self.next_print_wraps = true;
            }
        } else {
            if self.insert_mode {
                self.buffer.insert(self.cursor.position(), 1, cell);
            } else {
                self.buffer.print(self.cursor.position(), cell);
            }
            self.do_move_cursor_to_col(next_col);
        }
        self.dirty_lines.add(self.cursor.row);
    }

    pub fn execute(&mut self, input: char) {
        match input {
            '\u{08}' => {
                if self.next_print_wraps {
                    self.move_cursor_to_rel_col(-2);
                } else {
                    self.move_cursor_to_rel_col(-1);
                }
            }
            '\u{09}' => self.move_cursor_to_next_tab(1),
            '\u{0a}' | '\u{0b}' | '\u{0c}' | '\u{84}' => {
                self.move_cursor_down_with_scroll();
                if self.new_line_mode {
                    self.do_move_cursor_to_col(0);
                }
            }
            '\u{0d}' => self.do_move_cursor_to_col(0),
            '\u{0e}' => self.active_charset = 1,
            '\u{0f}' => self.active_charset = 0,
            '\u{85}' => {
                self.move_cursor_down_with_scroll();
                self.do_move_cursor_to_col(0);
            }
            '\u{88}' => self.set_tab(),
            '\u{8d}' => {
                if self.cursor.row == self.top_margin {
                    self.scroll_down_in_region(1);
                } else if self.cursor.row > 0 {
                    self.move_cursor_to_row(self.cursor.row - 1);
                }
            }
            _ => panic!("Unhandled execute: {}", input),
        }
    }

    // https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
    pub fn csi_dispatch(&mut self, params: &Params, intermediates: &[char], input: char) {
        match (input, intermediates) {
            ('@', []) => {
                self.buffer.insert(
                    self.cursor.position(),
                    params.get(0, 1),
                    Cell::blank(self.pen.clone()),
                );
                self.dirty_lines.add(self.cursor.row);
            }
            ('A', []) => self.cursor_up(params.get(0, 1)),
            ('B', []) | ('e', []) => self.cursor_down(params.get(0, 1)),
            ('b', []) => {
                if self.cursor.col > 0 {
                    let n = params.get(0, 1);
                    let char = self.buffer.view()[self.cursor.row].cells[self.cursor.col - 1].0;
                    for _ in 0..n {
                        self.print(char);
                    }
                }
            }
            ('C', []) | ('a', []) => self.move_cursor_to_rel_col(params.get(0, 1) as isize),
            ('D', []) => {
                let mut rel_col = -(params.get(0, 1) as isize);
                if self.next_print_wraps {
                    rel_col -= 1;
                }
                self.move_cursor_to_rel_col(rel_col);
            }
            ('d', []) => self.move_cursor_to_row(params.get(0, 1) - 1),
            ('E', []) => {
                self.cursor_down(params.get(0, 1));
                self.do_move_cursor_to_col(0);
            }
            ('F', []) => {
                self.cursor_up(params.get(0, 1));
                self.do_move_cursor_to_col(0);
            }
            ('G', []) | ('`', []) => self.move_cursor_to_col(params.get(0, 1) - 1),
            ('g', []) => match params.get(0, 0) {
                0 => self.clear_tab(),
                3 => self.clear_all_tabs(),
                _ => (),
            },
            ('H', []) | ('f', []) => {
                self.move_cursor_to_col(params.get(1, 1) - 1);
                self.move_cursor_to_row(params.get(0, 1) - 1);
            }
            ('h', []) => {
                for param in params.iter() {
                    match param {
                        4 => self.insert_mode = true,
                        20 => self.new_line_mode = true,
                        _ => (),
                    }
                }
            }
            ('h', ['?']) => {
                for param in params.iter() {
                    match param {
                        6 => {
                            self.origin_mode = true;
                            self.move_cursor_home();
                        }
                        7 => self.auto_wrap_mode = true,
                        25 => self.cursor.visible = true,
                        47 => {
                            self.switch_to_alternate_buffer();
                            self.reflow();
                        }
                        1047 => {
                            self.switch_to_alternate_buffer();
                            self.reflow();
                        }
                        1048 => self.save_cursor(),
                        1049 => {
                            self.save_cursor();
                            self.switch_to_alternate_buffer();
                            self.reflow();
                        }
                        _ => (),
                    }
                }
            }
            ('I', []) => self.move_cursor_to_next_tab(params.get(0, 1)),
            ('J', []) => match params.get(0, 0) {
                0 => {
                    self.buffer.erase(
                        self.cursor.position(),
                        EraseMode::FromCursorToEndOfView,
                        &self.pen,
                    );
                    self.dirty_lines.extend(self.cursor.row..self.rows);
                }
                1 => {
                    self.buffer.erase(
                        self.cursor.position(),
                        EraseMode::FromStartOfViewToCursor,
                        &self.pen,
                    );
                    self.dirty_lines.extend(0..self.cursor.row + 1);
                }
                2 => {
                    self.buffer
                        .erase(self.cursor.position(), EraseMode::WholeView, &self.pen);
                    self.dirty_lines.extend(0..self.rows);
                }
                _ => (),
            },
            ('K', []) => match params.get(0, 0) {
                0 => {
                    self.buffer.erase(
                        self.cursor.position(),
                        EraseMode::FromCursorToEndOfLine,
                        &self.pen,
                    );
                    self.dirty_lines.add(self.cursor.row);
                }
                1 => {
                    self.buffer.erase(
                        self.cursor.position(),
                        EraseMode::FromStartOfLineToCursor,
                        &self.pen,
                    );
                    self.dirty_lines.add(self.cursor.row);
                }
                2 => {
                    self.buffer
                        .erase(self.cursor.position(), EraseMode::WholeLine, &self.pen);
                    self.dirty_lines.add(self.cursor.row);
                }
                _ => (),
            },
            ('L', []) => {
                let range = if self.cursor.row <= self.bottom_margin {
                    self.cursor.row..self.bottom_margin + 1
                } else {
                    self.cursor.row..self.rows
                };
                self.buffer
                    .scroll_down(range.clone(), params.get(0, 1), &self.pen);
                self.dirty_lines.extend(range);
            }
            ('l', []) => {
                for param in params.iter() {
                    match param {
                        4 => self.insert_mode = false,
                        20 => self.new_line_mode = false,
                        _ => (),
                    }
                }
            }
            ('l', ['?']) => {
                for param in params.iter() {
                    match param {
                        6 => {
                            self.origin_mode = false;
                            self.move_cursor_home();
                        }
                        7 => self.auto_wrap_mode = false,
                        25 => self.cursor.visible = false,
                        47 => {
                            self.switch_to_primary_buffer();
                            self.reflow();
                        }
                        1047 => {
                            self.switch_to_primary_buffer();
                            self.reflow();
                        }
                        1048 => self.restore_cursor(),
                        1049 => {
                            self.switch_to_primary_buffer();
                            self.restore_cursor();
                            self.reflow();
                        }
                        _ => (),
                    }
                }
            }
            ('M', []) => {
                let range = if self.cursor.row <= self.bottom_margin {
                    self.cursor.row..self.bottom_margin + 1
                } else {
                    self.cursor.row..self.rows
                };
                self.buffer
                    .scroll_up(range.clone(), params.get(0, 1), &self.pen);
                self.dirty_lines.extend(range);
            }
            ('m', []) => {
                let mut ps = params.as_slice();
                while let Some(param) = ps.first() {
                    match param {
                        0 => {
                            self.pen = Pen::default();
                            ps = &ps[1..];
                        }
                        1 => {
                            self.pen.intensity = Intensity::Bold;
                            ps = &ps[1..];
                        }
                        2 => {
                            self.pen.intensity = Intensity::Faint;
                            ps = &ps[1..];
                        }
                        3 => {
                            self.pen.set_italic();
                            ps = &ps[1..];
                        }
                        4 => {
                            self.pen.set_underline();
                            ps = &ps[1..];
                        }
                        5 => {
                            self.pen.set_blink();
                            ps = &ps[1..];
                        }
                        7 => {
                            self.pen.set_inverse();
                            ps = &ps[1..];
                        }
                        9 => {
                            self.pen.set_strikethrough();
                            ps = &ps[1..];
                        }
                        21 | 22 => {
                            self.pen.intensity = Intensity::Normal;
                            ps = &ps[1..];
                        }
                        23 => {
                            self.pen.unset_italic();
                            ps = &ps[1..];
                        }
                        24 => {
                            self.pen.unset_underline();
                            ps = &ps[1..];
                        }
                        25 => {
                            self.pen.unset_blink();
                            ps = &ps[1..];
                        }
                        27 => {
                            self.pen.unset_inverse();
                            ps = &ps[1..];
                        }
                        param if *param >= 30 && *param <= 37 => {
                            self.pen.foreground = Some(Color::Indexed((param - 30) as u8));
                            ps = &ps[1..];
                        }
                        38 => match ps.get(1) {
                            None => {
                                ps = &ps[1..];
                            }
                            Some(2) => {
                                if let Some(b) = ps.get(4) {
                                    let r = ps.get(2).unwrap();
                                    let g = ps.get(3).unwrap();
                                    self.pen.foreground =
                                        Some(Color::RGB(RGB8::new(*r as u8, *g as u8, *b as u8)));
                                    ps = &ps[5..];
                                } else {
                                    ps = &ps[2..];
                                }
                            }
                            Some(5) => {
                                if let Some(param) = ps.get(2) {
                                    self.pen.foreground = Some(Color::Indexed(*param as u8));
                                    ps = &ps[3..];
                                } else {
                                    ps = &ps[2..];
                                }
                            }
                            Some(_) => {
                                ps = &ps[1..];
                            }
                        },
                        39 => {
                            self.pen.foreground = None;
                            ps = &ps[1..];
                        }
                        param if *param >= 40 && *param <= 47 => {
                            self.pen.background = Some(Color::Indexed((param - 40) as u8));
                            ps = &ps[1..];
                        }
                        48 => match ps.get(1) {
                            None => {
                                ps = &ps[1..];
                            }
                            Some(2) => {
                                if let Some(b) = ps.get(4) {
                                    let r = ps.get(2).unwrap();
                                    let g = ps.get(3).unwrap();
                                    self.pen.background =
                                        Some(Color::RGB(RGB8::new(*r as u8, *g as u8, *b as u8)));
                                    ps = &ps[5..];
                                } else {
                                    ps = &ps[2..];
                                }
                            }
                            Some(5) => {
                                if let Some(param) = ps.get(2) {
                                    self.pen.background = Some(Color::Indexed(*param as u8));
                                    ps = &ps[3..];
                                } else {
                                    ps = &ps[2..];
                                }
                            }
                            Some(_) => {
                                ps = &ps[1..];
                            }
                        },
                        49 => {
                            self.pen.background = None;
                            ps = &ps[1..];
                        }
                        param if *param >= 90 && *param <= 97 => {
                            self.pen.foreground = Some(Color::Indexed((param - 90 + 8) as u8));
                            ps = &ps[1..];
                        }
                        param if *param >= 100 && *param <= 107 => {
                            self.pen.background = Some(Color::Indexed((param - 100 + 8) as u8));
                            ps = &ps[1..];
                        }
                        _ => {
                            ps = &ps[1..];
                        }
                    }
                }
            }
            ('P', []) => {
                if self.cursor.col >= self.cols {
                    self.move_cursor_to_col(self.cols - 1);
                }
                self.buffer
                    .delete(self.cursor.position(), params.get(0, 1), &self.pen);
                self.dirty_lines.add(self.cursor.row);
            }
            ('p', ['!']) => self.soft_reset(),
            ('r', []) => {
                let top = params.get(0, 1) - 1;
                let bottom = params.get(1, self.rows) - 1;
                if top < bottom && bottom < self.rows {
                    self.top_margin = top;
                    self.bottom_margin = bottom;
                }
                self.move_cursor_home();
            }
            ('S', []) => self.scroll_up_in_region(params.get(0, 1)),
            ('s', []) => self.save_cursor(),
            ('T', []) => self.scroll_down_in_region(params.get(0, 1)),
            ('t', []) => unimplemented!("Resizing is not supported"),
            ('u', []) => self.restore_cursor(),
            ('W', []) => match params.get(0, 0) {
                0 => self.set_tab(),
                2 => self.clear_tab(),
                5 => self.clear_all_tabs(),
                _ => (),
            },
            ('X', []) => {
                let n = params.get(0, 1);
                self.buffer
                    .erase(self.cursor.position(), EraseMode::NextChars(n), &self.pen);
                self.dirty_lines.add(self.cursor.row);
            }
            ('Z', []) => self.move_cursor_to_prev_tab(params.get(0, 1)),
            // identify_terminal
            ('c', _) => (),
            // set_modify_other_keys
            ('m', ['>']) => (),
            // report_private_mode
            ('p', ['?', '$']) => (),
            // set_cursor_style
            ('q', [' ']) => (),
            // report_keyboard_mode
            ('u', ['?']) => (),
            _ => panic!("Unhandled CSI dispatch: {} / {:?}", input, intermediates),
        }
    }

    // https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Controls-beginning-with-ESC
    pub fn esc_dispatch(&mut self, intermediates: &[char], input: char) {
        match (input, intermediates) {
            ('@'..='_', []) => self.execute(((input as u8) + 0x40) as char),
            ('7', []) => self.save_cursor(),
            ('8', []) => self.restore_cursor(),
            ('c', []) => self.hard_reset(),
            ('8', ['#']) => {
                for row in 0..self.rows {
                    for col in 0..self.cols {
                        self.buffer
                            .print((col, row), Cell('\u{45}', Pen::default()));
                    }
                    self.dirty_lines.add(row);
                }
            }
            ('0', ['(']) => self.charsets[0] = Charset::Drawing,
            (_, ['(']) => self.charsets[0] = Charset::Ascii,
            ('0', [')']) => self.charsets[1] = Charset::Drawing,
            (_, [')']) => self.charsets[1] = Charset::Ascii,
            // set_keypad_application_mode
            ('=', []) => (),
            // unset_keypad_application_mode
            ('>', []) => (),
            _ => panic!("Unhandled ESC dispatch: {} / {:?}", input, intermediates),
        }
    }
}
