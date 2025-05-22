mod dialog;
mod directory;

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use env_logger::Builder;
use log::{LevelFilter, info};

use directory::Reader;

fn main() -> Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();

    let start = Instant::now();
    let result = copy();
    let duration = start.elapsed();
    info!("duration: {:.2?}", duration);

    match result {
        Ok(n) => {
            info!("copied: {n} files");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn copy() -> Result<usize> {
    let src = dialog::directory("source");
    let dest = dialog::directory("destination");
    if src == dest {
        info!("skipping: source & destination are same");
        return Ok(0);
    }
    if dialog::cancel(format!("copy files\nfrom: {:?}\nto: {:?}", src, dest)) {
        info!("cancelled: directories");
        return Ok(0);
    }

    let reader = Reader::new(vec![".DS_Store".into()], vec!["._".into()]);
    let src = reader.get(src)?;
    let dest = reader.get(dest)?;

    // files present in source but missing in destination
    let files: Vec<PathBuf> = src.files.difference(&dest.files).cloned().collect();
    if files.is_empty() {
        info!("skipping: no new files");
        return Ok(0);
    }
    if dialog::cancel(format!("this will copy {} files", files.len())) {
        info!("cancelled: copy");
        return Ok(0);
    }

    for file in &files {
        let from = src.root.join(file);
        let to = dest.root.join(file);

        let parent = to.parent().unwrap();
        if !parent.exists() {
            info!("creating missing directory: {:?}", parent);
            fs::create_dir_all(parent)?;
        }

        info!("copying: {:?} -> {:?}", from, to);
        fs::copy(&from, &to)?;
    }
    Ok(files.len())
}
