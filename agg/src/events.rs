use anyhow::{bail, Context, Error, Result};
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct Event {
    pub time: f64,
    pub data: String,
}

impl FromStr for Event {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(s)?;

        let time = value[0].as_f64().context("Invalid Event Time")?;
        assert_eq!(value[1].as_str(), Some("o"));
        let data = match value[2].as_str() {
            Some(data) => data.to_string(),
            None => bail!("Invalid Event Data"),
        };

        Ok(Self { time, data })
    }
}

#[derive(Debug)]
struct Batch<I> {
    iter: I,
    prev_time: f64,
    prev_data: String,
    max_frame_time: f64,
}

impl<I> Batch<I> {
    fn new(iter: I, fps_cap: u8) -> Self {
        Self {
            iter,
            prev_data: "".to_owned(),
            prev_time: 0.0,
            max_frame_time: 1.0 / (fps_cap as f64),
        }
    }
}

impl<I: Iterator<Item = Event>> Iterator for Batch<I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Self::Item { time, data }) => {
                if time - self.prev_time < self.max_frame_time {
                    self.prev_data.push_str(&data);
                    self.next()
                } else if !self.prev_data.is_empty() || self.prev_time == 0.0 {
                    let prev_time = self.prev_time;
                    self.prev_time = time;
                    let prev_data = std::mem::replace(&mut self.prev_data, data);
                    Some(Self::Item {
                        time: prev_time,
                        data: prev_data,
                    })
                } else {
                    self.prev_time = time;
                    self.prev_data = data;
                    self.next()
                }
            }
            None => {
                if !self.prev_data.is_empty() {
                    let prev_time = self.prev_time;
                    let prev_data = std::mem::replace(&mut self.prev_data, "".to_owned());
                    Some(Event {
                        time: prev_time,
                        data: prev_data,
                    })
                } else {
                    None
                }
            }
        }
    }
}

pub fn batch(iter: impl Iterator<Item = Event>, fps_cap: u8) -> impl Iterator<Item = Event> {
    Batch::new(iter, fps_cap)
}

pub fn accelerate(events: impl Iterator<Item = Event>, speed: f64) -> impl Iterator<Item = Event> {
    events.map(move |Event { time, data }| Event {
        time: time / speed,
        data,
    })
}
