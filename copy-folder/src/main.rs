mod dialog;
mod directory;

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use env_logger::Builder;
use log::{LevelFilter, info};
use rayon::prelude::*;

use directory::Reader;

const GB: u64 = 1_024u64.pow(3);

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

    let reader = Reader::new(vec![".DS_Store".into()], vec!["._".into()]);
    let src = reader.get(src)?;
    let dest = reader.get(dest)?;

    // files present in source but missing in destination
    let files: Vec<PathBuf> = src.files.difference(&dest.files).cloned().collect();
    if files.is_empty() {
        info!("skipping: no new files");
        return Ok(());
    }
    if dialog::cancel(format!("this will copy {} files", files.len())) {
        info!("cancelled: copy");
        return Ok(());
    }

    let start = Instant::now();

    // create any missing parent directories so file copying is independent
    let parents: HashSet<PathBuf> = files
        .iter()
        .map(|file| dest.root.join(file))
        .map(|path| path.parent().unwrap().to_path_buf())
        .collect();
    for parent in parents {
        if !parent.exists() {
            info!("creating missing directory: {:?}", parent);
            fs::create_dir_all(parent)?;
        }
    }

    // do the actual copying in parallel
    let gb: f64 = files
        .par_iter()
        .map(|file| {
            let from = src.root.join(file);
            let to = dest.root.join(file);
            info!("copying: {:?} -> {:?}", from, to);
            fs::copy(&from, &to).unwrap();

            let metadata = fs::metadata(&to).unwrap();
            metadata.len() as f64 / GB as f64
        })
        .sum();

    info!("duration: {:.2?}", start.elapsed());
    info!("copied: {} files", files.len());
    info!("copied: {:.4} gigabytes", gb);

    Ok(())
}
