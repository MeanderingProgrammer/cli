use crate::line::Line;
use crate::parser::Parser;
use crate::terminal::Terminal;

#[derive(Debug)]
pub struct Vt {
    parser: Parser,
    terminal: Terminal,
}

impl Vt {
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            parser: Parser::default(),
            terminal: Terminal::new(size),
        }
    }

    pub fn feed_str(&mut self, s: &str) -> Vec<usize> {
        self.parser.feed_str(s, &mut self.terminal);
        self.terminal.changes()
    }

    pub fn cursor(&self) -> Option<(usize, usize)> {
        self.terminal.cursor()
    }

    pub fn view(&self) -> &[Line] {
        self.terminal.view()
    }
}
