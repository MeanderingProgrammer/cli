use anyhow::Result;
use env_logger::Builder;
use log::{LevelFilter, info};

use copy_folder::{dialog, reader::Reader, transfer::Transfer};

fn main() -> Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();

    let src = dialog::dir("source");
    let dest = dialog::dir("destination");
    if src == dest {
        info!("skipping: source & destination are same");
        return Ok(());
    }
    if dialog::cancel(format!("copy files\nfrom: {:?}\nto: {:?}", src, dest)) {
        info!("cancelled: directories");
        return Ok(());
    }

    let reader = Reader::new([".DS_Store".into()], ["._".into()]);
    let src_files = reader.get(&src)?;
    let dest_files = reader.get(&dest)?;

    // files present in source but missing in destination
    let files: Vec<_> = src_files.difference(&dest_files).cloned().collect();
    if files.is_empty() {
        info!("skipping: no new files to copy");
        return Ok(());
    }
    if dialog::cancel(format!("this will copy {} files", files.len())) {
        info!("cancelled: copy");
        return Ok(());
    }

    Transfer::new(&src, &dest).run(files)
}
