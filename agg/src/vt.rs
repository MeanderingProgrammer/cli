use crate::events::Event;
use avt::{Cell, Vt};

#[derive(Debug)]
pub struct Frame {
    pub lines: Vec<Vec<Cell>>,
    pub cursor: Option<(usize, usize)>,
}

pub fn frames(
    events: impl Iterator<Item = Event>,
    size: (usize, usize),
) -> impl Iterator<Item = (f64, Frame)> {
    let mut vt = Vt::new(size);

    let mut prev_cursor = None;
    events.filter_map(move |Event { time, data }| {
        let changed_lines = vt.feed_str(&data);
        let cursor = vt.cursor();
        if changed_lines || cursor != prev_cursor {
            prev_cursor = cursor;
            let lines = vt.view().iter().map(|line| line.cells.clone()).collect();
            Some((time, Frame { lines, cursor }))
        } else {
            prev_cursor = cursor;
            log::debug!("skipping frame with no visual changes: {:?}", data);
            None
        }
    })
}
