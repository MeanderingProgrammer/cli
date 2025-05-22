mod dialog;
mod directory;

use std::time::Instant;

use anyhow::Result;

use directory::Reader;

fn main() -> Result<()> {
    let start = Instant::now();
    let result = copy();
    let duration = start.elapsed();
    println!("duration: {:.2?}", duration);
    match result {
        Ok(n) => {
            println!("files copied: {n}");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn copy() -> Result<usize> {
    let src = dialog::directory("source");
    let dest = dialog::directory("destination");
    if src == dest {
        println!("skipping: source & destination are same");
        return Ok(0);
    }
    if dialog::cancel(format!("copy files\nfrom: {:?}\nto: {:?}", src, dest)) {
        println!("skipping: cancelled directories");
        return Ok(0);
    }

    let reader = Reader::new(vec![".DS_Store".into()], vec!["._".into()]);
    let src = reader.get(src)?;
    let dest = reader.get(dest)?;

    // files present in source but missing in destination
    let files = dest.missing(&src);
    if files.is_empty() {
        println!("skipping: no new files");
        return Ok(0);
    }
    if dialog::cancel(format!("this will copy {} files", files.len())) {
        println!("skipping: cancelled copy");
        return Ok(0);
    }

    for file in &files {
        dest.copy(&src, file)?;
    }
    Ok(files.len())
}
