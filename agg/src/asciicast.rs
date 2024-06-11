use crate::events::TypedEvent;
use crate::theme::Theme;
use anyhow::{bail, Context, Error, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
pub struct V2Theme {
    fg: String,
    bg: String,
    palette: String,
}

#[derive(Debug, Deserialize)]
pub struct V2Header {
    pub width: usize,
    pub height: usize,
    pub idle_time_limit: Option<f64>,
    pub theme: Option<V2Theme>,
}

#[derive(Debug)]
pub struct Header {
    pub terminal_size: (usize, usize),
    pub idle_time_limit: Option<f64>,
    pub theme: Option<Theme>,
}

impl TryInto<Header> for V2Header {
    type Error = Error;

    fn try_into(self) -> Result<Header> {
        let theme = match self.theme {
            Some(V2Theme { bg, fg, palette })
                if bg.len() == 7
                    && fg.len() == 7
                    && (palette.len() == 63 || palette.len() == 127) =>
            {
                let palette = palette
                    .split(':')
                    .map(|s| &s[1..])
                    .collect::<Vec<_>>()
                    .join(",");

                let theme = format!("{},{},{}", &bg[1..], &fg[1..], palette);
                Some(theme.parse()?)
            }
            Some(_) => bail!("Invalid Theme"),
            None => None,
        };

        Ok(Header {
            terminal_size: (self.width, self.height),
            idle_time_limit: self.idle_time_limit,
            theme,
        })
    }
}

pub fn open(file: File) -> Result<(Header, impl Iterator<Item = Result<TypedEvent>>)> {
    let mut lines = BufReader::new(file).lines();

    let first_line = lines.next().context("Empty File")??;
    let v2_header: V2Header = serde_json::from_str(&first_line)?;
    let header: Header = v2_header.try_into()?;

    let events = lines
        .filter(|line| line.as_ref().map_or(true, |l| !l.is_empty()))
        .map(|line| line.map(|l| l.parse())?);

    Ok((header, events))
}
