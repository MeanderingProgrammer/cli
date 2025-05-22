use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use log::info;
use rayon::prelude::*;

#[derive(Debug)]
pub struct Transfer {
    src: PathBuf,
    dest: PathBuf,
}

impl Transfer {
    const GB: u64 = 1_024u64.pow(3);

    pub fn new(src: &Path, dest: &Path) -> Self {
        Self {
            src: src.to_path_buf(),
            dest: dest.to_path_buf(),
        }
    }

    pub fn run<I>(&self, files: I) -> Result<()>
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let files: HashSet<_> = files.into_iter().collect();

        // create any missing parent directories so file copying is independent
        let parents: HashSet<_> = files
            .iter()
            .map(|file| self.dest.join(file))
            .map(|path| path.parent().unwrap().to_path_buf())
            .collect();
        for parent in parents {
            if !parent.exists() {
                info!("creating directory: {:?}", parent);
                fs::create_dir_all(parent)?;
            }
        }

        // do the actual copying in parallel
        let start = Instant::now();
        let results = files.into_par_iter().map(|file| self.copy(&file)).collect();
        Self::summarize(start, results);
        Ok(())
    }

    fn copy(&self, file: &Path) -> Result<u64> {
        let from = self.src.join(file);
        let to = self.dest.join(file);
        // skip file if it already exist in destination
        if to.exists() {
            return Ok(0);
        }
        info!("copying: {:?} -> {:?}", from, to);
        fs::copy(&from, &to)?;
        self.cleanup(&to)?;
        Ok(fs::metadata(to)?.len())
    }

    #[cfg(target_os = "macos")]
    fn cleanup(&self, path: &Path) -> Result<()> {
        let parent = path.parent().unwrap().to_path_buf();
        let name = path.file_name().unwrap().to_str().unwrap();
        let metadata = parent.join(format!("._{name}"));
        if metadata.exists() {
            info!("deleting: {:?}", metadata);
            fs::remove_file(metadata)?;
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    fn cleanup(&self, _path: &Path) -> Result<()> {
        // currently outside of macos no cleanup is needed
        Ok(())
    }

    fn summarize(start: Instant, results: Vec<Result<u64>>) {
        let mut successes = 0;
        let mut failures = 0;
        let mut gb = 0.0;
        for result in results {
            match result {
                Ok(bytes) => {
                    successes += 1;
                    gb += bytes as f64 / Self::GB as f64;
                }
                Err(_) => {
                    failures += 1;
                }
            }
        }
        info!("duration: {:.2?}", start.elapsed());
        info!("successes: {}", successes);
        info!("failures: {}", failures);
        info!("gigabytes: {:.4}", gb);
    }
}
