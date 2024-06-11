use anyhow::{bail, Context, Error, Result};
use std::str::FromStr;

#[derive(PartialEq, Eq, Debug)]
pub enum EventType {
    Output,
    Input,
    Other(char),
}

#[derive(Debug)]
pub struct Event {
    pub time: f64,
    pub data: String,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            time: 0.0,
            data: "".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct TypedEvent {
    pub event: Event,
    pub event_type: EventType,
}

impl FromStr for TypedEvent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(s)?;

        let time = value[0].as_f64().context("Invalid Event Time")?;

        let event_type = match value[1].as_str() {
            Some("o") => EventType::Output,
            Some("i") => EventType::Input,
            Some(s) => {
                if !s.is_empty() {
                    EventType::Other(s.chars().next().unwrap())
                } else {
                    bail!("Invalid Event Type")
                }
            }
            None => bail!("Invalid Event Type"),
        };

        let data = match value[2].as_str() {
            Some(data) => data.to_owned(),
            None => bail!("Invalid Event Data"),
        };

        let event = Event { time, data };
        Ok(Self { event, event_type })
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
            Some(Event { time, data }) => {
                if time - self.prev_time < self.max_frame_time {
                    self.prev_data.push_str(&data);
                    self.next()
                } else if !self.prev_data.is_empty() || self.prev_time == 0.0 {
                    let prev_time = self.prev_time;
                    self.prev_time = time;
                    let prev_data = std::mem::replace(&mut self.prev_data, data);
                    Some(Event {
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

pub fn stdout(events: impl Iterator<Item = Result<TypedEvent>>) -> impl Iterator<Item = Event> {
    events.filter_map(|e| match e {
        Ok(TypedEvent {
            event,
            event_type: EventType::Output,
        }) => Some(event),
        _ => None,
    })
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

pub fn limit_idle(events: impl Iterator<Item = Event>, limit: f64) -> impl Iterator<Item = Event> {
    let mut prev_time = 0.0;
    let mut offset = 0.0;
    events.map(move |Event { time, data }| {
        let delay = time - prev_time;
        let excess = delay - limit;
        if excess > 0.0 {
            offset += excess;
        }
        prev_time = time;
        Event {
            time: time - offset,
            data,
        }
    })
}
