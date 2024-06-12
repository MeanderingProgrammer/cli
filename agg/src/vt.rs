use crate::events::Event;
use avt::{Pen, Vt};

#[derive(Debug)]
pub struct Frame {
    pub lines: Vec<Vec<(char, Pen)>>,
    pub cursor: Option<(usize, usize)>,
}

pub fn frames(
    events: impl Iterator<Item = Event>,
    terminal_size: (usize, usize),
) -> impl Iterator<Item = (f64, Frame)> {
    let mut vt = Vt::builder()
        .size(terminal_size.0, terminal_size.1)
        .scrollback_limit(0)
        .build();

    let mut prev_cursor = None;
    events.filter_map(move |Event { time, data }| {
        let (changed_lines, _) = vt.feed_str(&data);
        let cursor: Option<(usize, usize)> = vt.cursor().into();
        if !changed_lines.is_empty() || cursor != prev_cursor {
            prev_cursor = cursor;
            let lines = vt
                .view()
                .iter()
                .map(|line| line.cells().collect())
                .collect();
            Some((time, Frame { lines, cursor }))
        } else {
            prev_cursor = cursor;
            log::debug!("skipping frame with no visual changes: {:?}", data);
            None
        }
    })
}
