use crate::buffer::{Buffer, EraseMode};
use crate::cell::Cell;
use crate::charset::Charset;
use crate::cursor::Cursor;
use crate::line::Line;
use crate::parser::Emulator;
use crate::pen::Intensity;
use crate::tabs::Tabs;
use crate::Pen;
use std::cmp::{max, min};

#[derive(Debug, Default)]
enum BufferType {
    #[default]
    Primary,
    Alternate,
}

#[derive(Debug, Default)]
enum CursorKeys {
    #[default]
    Normal,
    Application,
}

#[derive(Debug, Clone, Default)]
enum OriginMode {
    #[default]
    Absolute,
    Relative,
}

#[derive(Debug)]
struct SavedCtx {
    cursor_col: usize,
    cursor_row: usize,
    pen: Pen,
    origin_mode: OriginMode,
    auto_wrap_mode: bool,
}

impl Default for SavedCtx {
    fn default() -> Self {
        Self {
            cursor_col: 0,
            cursor_row: 0,
            pen: Pen::default(),
            origin_mode: OriginMode::default(),
            auto_wrap_mode: true,
        }
    }
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
    cursor_keys: CursorKeys,
    origin_mode: OriginMode,
    auto_wrap_mode: bool,
    new_line_mode: bool,
    next_print_wraps: bool,
    top_margin: usize,
    bottom_margin: usize,
    saved_ctx: SavedCtx,
    alternate_saved_ctx: SavedCtx,
    dirty: bool,
}

impl Terminal {
    pub fn new((cols, rows): (usize, usize)) -> Self {
        Self {
            cols,
            rows,
            buffer: Buffer::new(cols, rows, Pen::default()),
            other_buffer: Buffer::new(cols, rows, Pen::default()),
            active_buffer_type: BufferType::default(),
            cursor: Cursor::default(),
            pen: Pen::default(),
            charsets: [Charset::default(), Charset::default()],
            active_charset: 0,
            tabs: Tabs::new(cols),
            insert_mode: false,
            cursor_keys: CursorKeys::default(),
            origin_mode: OriginMode::default(),
            auto_wrap_mode: true,
            new_line_mode: false,
            next_print_wraps: false,
            top_margin: 0,
            bottom_margin: (rows - 1),
            saved_ctx: SavedCtx::default(),
            alternate_saved_ctx: SavedCtx::default(),
            dirty: false,
        }
    }

    pub fn changes(&mut self) -> bool {
        let result = self.dirty;
        self.dirty = false;
        result
    }

    pub fn cursor(&self) -> Option<(usize, usize)> {
        self.cursor.clone().into()
    }

    pub fn view(&self) -> &[Line] {
        self.buffer.view()
    }
}

// https://github.com/alacritty/vte/blob/master/src/ansi.rs
// https://github.com/alacritty/alacritty/blob/master/alacritty_terminal/src/term/mod.rs
// https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
// https://en.wikipedia.org/wiki/ANSI_escape_code
impl Emulator for Terminal {
    fn print(&mut self, input: char) {
        let input = self.charsets[self.active_charset].map(input);
        let cell = Cell::new(input, self.pen.clone());
        if self.auto_wrap_mode && self.next_print_wraps {
            self.carriage_return();
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
        self.dirty = true;
    }

    fn execute(&mut self, input: char) {
        match input {
            '\u{08}' => {
                if self.next_print_wraps {
                    self.move_cursor_to_rel_col(-2);
                } else {
                    self.move_cursor_to_rel_col(-1);
                }
            }
            '\u{09}' => self.move_cursor_to_next_tab(1),
            '\u{0a}' | '\u{0b}' | '\u{0c}' => self.linefeed(),
            '\u{0d}' => self.carriage_return(),
            '\u{0e}' => self.active_charset = 1,
            '\u{0f}' => self.active_charset = 0,
            _ => panic!("Unhandled execute: {:x}", input as u32),
        }
    }

    fn csi_dispatch(&mut self, input: char, intermediates: &[char], params: &[u16]) {
        let mut params_iter = params.iter();
        let mut next_param = |default: usize| match params_iter.next() {
            Some(&param) if param != 0 => param as usize,
            _ => default,
        };

        match (input, intermediates) {
            ('@', []) => {
                let cell = Cell::blank(self.pen.clone());
                self.buffer
                    .insert(self.cursor.position(), next_param(1), cell);
                self.dirty = true;
            }
            ('A', []) => self.cursor_up(next_param(1)),
            ('B', []) | ('e', []) => self.cursor_down(next_param(1)),
            ('b', []) => {
                assert!(self.cursor.col > 0);
                let char = self.buffer.view()[self.cursor.row].cells[self.cursor.col - 1].ch;
                for _ in 0..next_param(1) {
                    self.print(char);
                }
            }
            ('C', []) | ('a', []) => self.move_cursor_to_rel_col(next_param(1) as isize),
            ('D', []) => {
                let mut rel_col = -(next_param(1) as isize);
                if self.next_print_wraps {
                    rel_col -= 1;
                }
                self.move_cursor_to_rel_col(rel_col);
            }
            ('d', []) => self.move_cursor_to_row(next_param(1) - 1),
            ('E', []) => {
                self.cursor_down(next_param(1));
                self.carriage_return();
            }
            ('F', []) => {
                self.cursor_up(next_param(1));
                self.carriage_return();
            }
            ('G', []) | ('`', []) => self.move_cursor_to_col(next_param(1) - 1),
            ('g', []) => match next_param(0) {
                0 => self.clear_tab(),
                3 => self.clear_all_tabs(),
                param => log::debug!("Unhandled 'g' param: {:?}", param),
            },
            ('H', []) | ('f', []) => {
                let x = next_param(1);
                let y = next_param(1);
                self.move_cursor_to_col(y - 1);
                self.move_cursor_to_row(x - 1);
            }
            ('h', []) => {
                for param in params.iter() {
                    match param {
                        4 => self.insert_mode = true,
                        20 => self.new_line_mode = true,
                        // Normal cursor visibility
                        34 => (),
                        _ => log::debug!("Unhandled 'h' param: {:?}", param),
                    }
                }
            }
            ('l', []) => {
                for param in params_iter {
                    match param {
                        4 => self.insert_mode = false,
                        20 => self.new_line_mode = false,
                        _ => log::debug!("Unhandled 'l' param: {:?}", param),
                    }
                }
            }
            ('h', ['?']) => {
                for param in params_iter {
                    match param {
                        1 => self.cursor_keys = CursorKeys::Application,
                        6 => {
                            self.origin_mode = OriginMode::Relative;
                            self.move_cursor_home();
                        }
                        7 => self.auto_wrap_mode = true,
                        25 => self.cursor.visible = true,
                        47 | 1047 => self.switch_to_alternate_buffer(false),
                        1048 => self.save_cursor(),
                        1049 => self.switch_to_alternate_buffer(true),
                        _ => log::debug!("Unhandled 'h?' param: {:?}", param),
                    }
                }
            }
            ('l', ['?']) => {
                for param in params.iter() {
                    match param {
                        1 => self.cursor_keys = CursorKeys::Normal,
                        6 => {
                            self.origin_mode = OriginMode::Absolute;
                            self.move_cursor_home();
                        }
                        7 => self.auto_wrap_mode = false,
                        25 => self.cursor.visible = false,
                        47 | 1047 => self.switch_to_primary_buffer(false),
                        1048 => self.restore_cursor(),
                        1049 => self.switch_to_primary_buffer(true),
                        _ => log::debug!("Unhandled 'l?' param: {:?}", param),
                    }
                }
            }
            ('I', []) => self.move_cursor_to_next_tab(next_param(1)),
            ('J', []) => match next_param(0) {
                0 => self.erase(EraseMode::FromCursorToEndOfView),
                1 => self.erase(EraseMode::FromStartOfViewToCursor),
                2 => self.erase(EraseMode::WholeView),
                param => log::debug!("Unhandled 'J' param: {:?}", param),
            },
            ('K', []) => match next_param(0) {
                0 => self.erase(EraseMode::FromCursorToEndOfLine),
                1 => self.erase(EraseMode::FromStartOfLineToCursor),
                2 => self.erase(EraseMode::WholeLine),
                param => log::debug!("Unhandled 'K' param: {:?}", param),
            },
            ('L', []) => {
                let range = if self.cursor.row <= self.bottom_margin {
                    self.cursor.row..self.bottom_margin + 1
                } else {
                    self.cursor.row..self.rows
                };
                self.buffer.scroll_down(range, next_param(1), &self.pen);
                self.dirty = true;
            }
            ('M', []) => {
                let range = if self.cursor.row <= self.bottom_margin {
                    self.cursor.row..self.bottom_margin + 1
                } else {
                    self.cursor.row..self.rows
                };
                self.buffer.scroll_up(range, next_param(1), &self.pen);
                self.dirty = true;
            }
            ('m', []) => match params {
                [0] => self.pen = Pen::default(),
                [1] => self.pen.intensity = Intensity::Bold,
                [0, 1] => {
                    self.pen = Pen::default();
                    self.pen.intensity = Intensity::Bold;
                }
                [2] => self.pen.intensity = Intensity::Faint,
                [3] => self.pen.set_italic(),
                [4] => self.pen.set_underline(),
                [5] => self.pen.set_blink(),
                [7] => self.pen.set_inverse(),
                [9] => self.pen.set_strikethrough(),
                [21..=22] => self.pen.intensity = Intensity::Normal,
                [23] => self.pen.unset_italic(),
                [24] => self.pen.unset_underline(),
                [25] => self.pen.unset_blink(),
                [27] => self.pen.unset_inverse(),
                [30..=37] => self.pen.foreground = Some((params[0] - 30).into()),
                [38, 2, r, g, b] => self.pen.foreground = Some((*r, *g, *b).into()),
                [38, 5, i] => self.pen.foreground = Some((*i).into()),
                [39] => self.pen.foreground = None,
                [40..=47] => self.pen.background = Some((params[0] - 40).into()),
                [48, 2, r, g, b] => self.pen.background = Some((*r, *g, *b).into()),
                [48, 5, i] => self.pen.background = Some((*i).into()),
                [49] => self.pen.background = None,
                [90..=97] => self.pen.foreground = Some((params[0] - 90 + 8).into()),
                [100..=107] => self.pen.background = Some((params[0] - 100 + 8).into()),
                _ => panic!("Unhandled 'm' params: {:?}", params),
            },
            ('P', []) => {
                if self.cursor.col >= self.cols {
                    self.move_cursor_to_col(self.cols - 1);
                }
                self.buffer
                    .delete(self.cursor.position(), next_param(1), &self.pen);
                self.dirty = true;
            }
            ('p', ['!']) => self.soft_reset(),
            ('r', []) => {
                let top = next_param(1) - 1;
                let bottom = next_param(self.rows) - 1;
                if top < bottom && bottom < self.rows {
                    self.top_margin = top;
                    self.bottom_margin = bottom;
                }
                self.move_cursor_home();
            }
            ('S', []) => self.scroll_up_in_region(next_param(1)),
            ('s', []) => self.save_cursor(),
            ('T', []) => self.scroll_down_in_region(next_param(1)),
            ('t', []) => panic!("Resizing is not supported"),
            ('u', []) => self.restore_cursor(),
            ('W', []) => match next_param(0) {
                0 => self.set_tab(),
                2 => self.clear_tab(),
                5 => self.clear_all_tabs(),
                param => log::debug!("Unhandled 'W' param: {:?}", param),
            },
            ('X', []) => self.erase(EraseMode::NextChars(next_param(1))),
            ('Z', []) => self.move_cursor_to_prev_tab(next_param(1)),
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
            _ => panic!("Unhandled CSI dispatch: {:?} / {:?}", input, intermediates),
        }
    }

    fn esc_dispatch(&mut self, input: char, intermediates: &[char]) {
        match (input, intermediates) {
            ('D', []) => self.linefeed(),
            ('E', []) => {
                self.linefeed();
                self.carriage_return();
            }
            ('H', []) => self.set_tab(),
            ('M', []) => {
                if self.cursor.row == self.top_margin {
                    self.scroll_down_in_region(1);
                } else if self.cursor.row > 0 {
                    self.move_cursor_to_row(self.cursor.row - 1);
                }
            }
            ('7', []) => self.save_cursor(),
            ('8', []) => self.restore_cursor(),
            ('c', []) => self.hard_reset(),
            ('8', ['#']) => {
                let cell: Cell = '\u{45}'.into();
                for row in 0..self.rows {
                    for col in 0..self.cols {
                        self.buffer.print((col, row), cell.clone());
                    }
                }
                self.dirty = true;
            }
            ('0', ['(']) => self.charsets[0] = Charset::Drawing,
            (_, ['(']) => self.charsets[0] = Charset::Ascii,
            ('0', [')']) => self.charsets[1] = Charset::Drawing,
            (_, [')']) => self.charsets[1] = Charset::Ascii,
            // String terminator do nothing
            ('\\', []) => (),
            // set_keypad_application_mode
            ('=', []) => (),
            // unset_keypad_application_mode
            ('>', []) => (),
            _ => panic!("Unhandled ESC dispatch: {:?} / {:?}", input, intermediates),
        }
    }
}

impl Terminal {
    fn erase(&mut self, mode: EraseMode) {
        self.buffer.erase(self.cursor.position(), mode, &self.pen);
        self.dirty = true;
    }

    fn linefeed(&mut self) {
        if self.cursor.row == self.bottom_margin {
            self.scroll_up_in_region(1);
        } else if self.cursor.row < self.rows - 1 {
            self.do_move_cursor_to_row(self.cursor.row + 1);
        }
        if self.new_line_mode {
            self.carriage_return();
        }
    }

    fn carriage_return(&mut self) {
        self.do_move_cursor_to_col(0);
    }

    fn move_cursor_home(&mut self) {
        self.carriage_return();
        self.do_move_cursor_to_row(self.actual_top_margin());
    }

    fn move_cursor_to_col(&mut self, col: usize) {
        if col >= self.cols {
            self.do_move_cursor_to_col(self.cols - 1);
        } else {
            self.do_move_cursor_to_col(col);
        }
    }

    fn move_cursor_to_rel_col(&mut self, rel_col: isize) {
        let new_col = self.cursor.col as isize + rel_col;
        if new_col < 0 {
            self.carriage_return();
        } else if new_col as usize >= self.cols {
            self.do_move_cursor_to_col(self.cols - 1);
        } else {
            self.do_move_cursor_to_col(new_col as usize);
        }
    }

    fn move_cursor_to_row(&mut self, row: usize) {
        let top = self.actual_top_margin();
        let bottom = self.actual_bottom_margin();
        let row = min(max(top + row, top), bottom);
        self.do_move_cursor_to_row(row);
    }

    fn cursor_down(&mut self, n: usize) {
        let new_y = if self.cursor.row > self.bottom_margin {
            min(self.rows - 1, self.cursor.row + n)
        } else {
            min(self.bottom_margin, self.cursor.row + n)
        };
        self.do_move_cursor_to_row(new_y);
    }

    fn cursor_up(&mut self, n: usize) {
        let mut new_y = (self.cursor.row as isize) - (n as isize);
        new_y = if self.cursor.row < self.top_margin {
            max(new_y, 0)
        } else {
            max(new_y, self.top_margin as isize)
        };
        self.do_move_cursor_to_row(new_y as usize);
    }

    fn do_move_cursor_to_col(&mut self, col: usize) {
        self.cursor.col = col;
        self.next_print_wraps = false;
    }

    fn do_move_cursor_to_row(&mut self, row: usize) {
        self.cursor.col = min(self.cursor.col, self.cols - 1);
        self.cursor.row = row;
        self.next_print_wraps = false;
    }

    fn actual_top_margin(&self) -> usize {
        match self.origin_mode {
            OriginMode::Absolute => 0,
            OriginMode::Relative => self.top_margin,
        }
    }

    fn actual_bottom_margin(&self) -> usize {
        match self.origin_mode {
            OriginMode::Absolute => self.rows - 1,
            OriginMode::Relative => self.bottom_margin,
        }
    }

    fn scroll_up_in_region(&mut self, n: usize) {
        let range = self.top_margin..self.bottom_margin + 1;
        self.buffer.scroll_up(range, n, &self.pen);
        self.dirty = true;
    }

    fn scroll_down_in_region(&mut self, n: usize) {
        let range = self.top_margin..self.bottom_margin + 1;
        self.buffer.scroll_down(range, n, &self.pen);
        self.dirty = true;
    }
}

impl Terminal {
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

    fn move_cursor_to_next_tab(&mut self, n: usize) {
        let next_tab = self.tabs.after(self.cursor.col, n).unwrap_or(self.cols - 1);
        self.move_cursor_to_col(next_tab);
    }

    fn move_cursor_to_prev_tab(&mut self, n: usize) {
        let prev_tab = self.tabs.before(self.cursor.col, n).unwrap_or(0);
        self.move_cursor_to_col(prev_tab);
    }
}

impl Terminal {
    fn switch_to_alternate_buffer(&mut self, save_cursor: bool) {
        if save_cursor {
            self.save_cursor();
        }
        if let BufferType::Primary = self.active_buffer_type {
            self.active_buffer_type = BufferType::Alternate;
            std::mem::swap(&mut self.saved_ctx, &mut self.alternate_saved_ctx);
            std::mem::swap(&mut self.buffer, &mut self.other_buffer);
            self.buffer = Buffer::new(self.cols, self.rows, self.pen.clone());
            self.dirty = true;
        }
    }

    fn save_cursor(&mut self) {
        self.saved_ctx.cursor_col = min(self.cursor.col, self.cols - 1);
        self.saved_ctx.cursor_row = self.cursor.row;
        self.saved_ctx.pen = self.pen.clone();
        self.saved_ctx.origin_mode = self.origin_mode.clone();
        self.saved_ctx.auto_wrap_mode = self.auto_wrap_mode;
    }

    fn switch_to_primary_buffer(&mut self, restore_cursor: bool) {
        if let BufferType::Alternate = self.active_buffer_type {
            self.active_buffer_type = BufferType::Primary;
            std::mem::swap(&mut self.saved_ctx, &mut self.alternate_saved_ctx);
            std::mem::swap(&mut self.buffer, &mut self.other_buffer);
            self.dirty = true;
        }
        if restore_cursor {
            self.restore_cursor();
        }
    }

    fn restore_cursor(&mut self) {
        self.cursor.col = self.saved_ctx.cursor_col;
        self.cursor.row = self.saved_ctx.cursor_row;
        self.pen = self.saved_ctx.pen.clone();
        self.origin_mode = self.saved_ctx.origin_mode.clone();
        self.auto_wrap_mode = self.saved_ctx.auto_wrap_mode;
        self.next_print_wraps = false;
    }

    fn soft_reset(&mut self) {
        self.cursor.visible = true;
        self.pen = Pen::default();
        self.charsets = [Charset::default(), Charset::default()];
        self.active_charset = 0;
        self.insert_mode = false;
        self.origin_mode = OriginMode::default();
        self.top_margin = 0;
        self.bottom_margin = self.rows - 1;
        self.saved_ctx = SavedCtx::default();
    }

    fn hard_reset(&mut self) {
        self.buffer = Buffer::new(self.cols, self.rows, Pen::default());
        self.other_buffer = Buffer::new(self.cols, self.rows, Pen::default());
        self.active_buffer_type = BufferType::default();
        self.cursor = Cursor::default();
        self.pen = Pen::default();
        self.charsets = [Charset::default(), Charset::default()];
        self.active_charset = 0;
        self.tabs = Tabs::new(self.cols);
        self.insert_mode = false;
        self.origin_mode = OriginMode::default();
        self.auto_wrap_mode = true;
        self.new_line_mode = false;
        self.next_print_wraps = false;
        self.top_margin = 0;
        self.bottom_margin = self.rows - 1;
        self.saved_ctx = SavedCtx::default();
        self.alternate_saved_ctx = SavedCtx::default();
        self.dirty = false;
    }
}
