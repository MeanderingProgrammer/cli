use crate::events::{self, Event};
use anyhow::{Context, Result};
use avt::{Cell, Vt};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct V2Header {
    width: usize,
    height: usize,
}

#[derive(Debug, PartialEq)]
pub struct Header {
    pub terminal_size: (usize, usize),
}

impl From<V2Header> for Header {
    fn from(val: V2Header) -> Self {
        Self {
            terminal_size: (val.width, val.height),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Frame {
    pub time: f64,
    pub cursor: Option<(usize, usize)>,
    pub lines: Vec<Vec<Cell>>,
}

impl Frame {
    fn new(time: f64, cursor: Option<(usize, usize)>, lines: Vec<Vec<Cell>>) -> Self {
        Self {
            time,
            cursor,
            lines,
        }
    }
}

#[derive(Debug)]
pub struct Reader {
    pub file: File,
    pub speed: f64,
    pub fps_cap: u8,
}

impl Reader {
    pub fn parse(self) -> Result<(Header, u64, impl Iterator<Item = Frame>)> {
        let mut lines = BufReader::new(self.file).lines();

        let first_line = lines.next().context("Empty File")??;
        let header = Self::parse_header(&first_line)?;

        let events = lines
            .map(|line| line.unwrap())
            .filter(|line| !line.is_empty())
            .filter_map(|line| line.parse().ok());

        let events = std::iter::once(Event::default()).chain(events);
        let events = events::accelerate(events, self.speed);
        let events = events::batch(events, self.fps_cap);
        let events: Vec<Event> = events.collect();

        let count = events.len();
        let frames = Self::frames(&header, events);

        Ok((header, count as u64, frames))
    }

    fn parse_header(line: &str) -> Result<Header> {
        let v2_header: V2Header = serde_json::from_str(line)?;
        Ok(v2_header.into())
    }

    fn frames(header: &Header, events: Vec<Event>) -> impl Iterator<Item = Frame> {
        let mut vt = Vt::new(header.terminal_size);
        let mut prev_cursor = None;
        events.into_iter().filter_map(move |Event { time, data }| {
            let changed_lines = vt.feed_str(&data);
            let cursor = vt.cursor();
            if changed_lines || cursor != prev_cursor {
                prev_cursor = cursor;
                let lines = vt.view().iter().map(|line| line.cells.clone()).collect();
                Some(Frame::new(time, cursor, lines))
            } else {
                prev_cursor = cursor;
                log::debug!("Skipping frame with no changes: {:?}", data);
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avt::{rgb::RGB8, Color, Intensity, Pen};

    #[test]
    fn frame_cast() {
        let reader = Reader {
            file: File::open("test/frame.cast").unwrap(),
            speed: 1.0,
            fps_cap: 30,
        };
        let (header, count, frames) = reader.parse().unwrap();
        let frames: Vec<Frame> = frames.collect();

        assert_eq!(
            header,
            Header {
                terminal_size: (55, 30)
            }
        );
        assert_eq!(count, 2);

        let base = Pen {
            foreground: Some(Color::RGB(RGB8::new(224, 222, 244))),
            background: Some(Color::RGB(RGB8::new(25, 23, 36))),
            intensity: Intensity::Normal,
            attrs: 0,
        };

        let title = Pen {
            foreground: Some(Color::RGB(RGB8::new(196, 167, 231))),
            background: Some(Color::RGB(RGB8::new(38, 35, 58))),
            intensity: Intensity::Bold,
            attrs: 0,
        };

        let empty = Pen {
            foreground: Some(Color::RGB(RGB8::new(0, 0, 0))),
            background: None,
            intensity: Intensity::Normal,
            attrs: 0,
        };

        let mut bold = base.clone();
        bold.intensity = Intensity::Bold;

        let mut title_bg = base.clone();
        title_bg.background.clone_from(&title.background);

        let mut sign = base.clone();
        sign.foreground = Some(Color::RGB(RGB8::new(110, 106, 134)));

        let mut text = base.clone();
        text.foreground = Some(Color::RGB(RGB8::new(144, 140, 170)));

        let mut link = base.clone();
        link.foreground = Some(Color::RGB(RGB8::new(156, 207, 216)));

        let frame_2_lines = vec![
            line(
                55,
                &title_bg,
                &[
                    ("1   ", &bold),
                    ("#", &title),
                    (" ", &title_bg),
                    ("Note", &title),
                ],
            ),
            line(55, &base, &[("  1 ", &sign)]),
            line(
                55,
                &base,
                &[
                    ("  2 ", &sign),
                    ("> [", &text),
                    ("!NOTE", &link),
                    ("]", &text),
                ],
            ),
            line(55, &base, &[("  3 ", &sign), ("> A regular note", &text)]),
            line(55, &base, &[("  4 ", &sign)]),
            line(55, &empty, &[("  5 ", &sign)]),
        ];

        assert_eq!(
            frames,
            [
                Frame::new(0.0, Some((0, 0)), vec![blank(55, &Pen::default()); 30]),
                Frame::new(
                    0.108509,
                    None,
                    [frame_2_lines, vec![blank(55, &empty); 24]].concat()
                ),
            ]
        );
    }

    fn line(width: usize, bg: &Pen, text: &[(&str, &Pen)]) -> Vec<Cell> {
        let result: Vec<Cell> = text
            .iter()
            .flat_map(|(s, pen)| s.chars().map(|ch| Cell::new(ch, pen)))
            .collect();
        let filler = width - result.len();
        [result, blank(filler, bg)].concat()
    }

    fn blank(n: usize, pen: &Pen) -> Vec<Cell> {
        vec![Cell::blank(pen); n]
    }
}
