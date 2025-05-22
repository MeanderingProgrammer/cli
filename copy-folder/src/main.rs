mod dialog;
mod reader;
mod transfer;

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use env_logger::Builder;
use log::{LevelFilter, info};

use reader::Reader;
use transfer::Transfer;

fn main() -> Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();

    let src = dialog::directory("source");
    let dest = dialog::directory("destination");
    if src == dest {
        info!("skipping: source & destination are same");
        return Ok(());
    }
    if dialog::cancel(format!("copy files\nfrom: {:?}\nto: {:?}", src, dest)) {
        info!("cancelled: directories");
        return Ok(());
    }

    let reader = Reader {
        names: HashSet::from([".DS_Store".into()]),
        prefixes: vec!["._".into()],
    };
    let src_files = reader.get(&src)?;
    let dest_files = reader.get(&dest)?;

    // files present in source but missing in destination
    let files: Vec<PathBuf> = src_files.difference(&dest_files).cloned().collect();
    if files.is_empty() {
        info!("skipping: no new files");
        return Ok(());
    }
    if dialog::cancel(format!("this will copy {} files", files.len())) {
        info!("cancelled: copy");
        return Ok(());
    }

    Transfer { src, dest }.run(files)?;

    Ok(())
}
