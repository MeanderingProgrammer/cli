use crate::events::Event;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct V2Header {
    width: usize,
    height: usize,
}

#[derive(Debug)]
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

pub fn open(file: File) -> Result<(Header, impl Iterator<Item = Event>)> {
    let mut lines = BufReader::new(file).lines();

    let first_line = lines.next().context("Empty File")??;
    let v2_header: V2Header = serde_json::from_str(&first_line)?;
    let header = v2_header.into();

    let events = lines
        .map(|line| line.unwrap())
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.parse().ok());

    Ok((header, events))
}
