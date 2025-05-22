use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use log::info;
use rayon::prelude::*;

#[derive(Debug)]
pub struct Transfer {
    pub src: PathBuf,
    pub dest: PathBuf,
}

impl Transfer {
    pub fn run(&self, files: Vec<PathBuf>) -> Result<()> {
        let mut summary = Summary {
            start: Instant::now(),
            successes: 0,
            failures: 0,
            gb: 0.0,
        };

        // create any missing parent directories so file copying is independent
        let parents: HashSet<PathBuf> = files
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
        let results: Vec<Result<u64>> =
            files.into_par_iter().map(|file| self.copy(&file)).collect();

        for result in results {
            summary.add(result);
        }

        summary.done();

        Ok(())
    }

    fn copy(&self, file: &Path) -> Result<u64> {
        let from = self.src.join(file);
        let to = self.dest.join(file);
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
        Ok(())
    }
}

#[derive(Debug)]
struct Summary {
    start: Instant,
    successes: usize,
    failures: usize,
    gb: f64,
}

impl Summary {
    const GB: u64 = 1_024u64.pow(3);

    fn add(&mut self, result: Result<u64>) {
        match result {
            Ok(bytes) => {
                self.successes += 1;
                self.gb += bytes as f64 / Self::GB as f64;
            }
            Err(_) => {
                self.failures += 1;
            }
        }
    }

    fn done(&self) {
        info!("duration: {:.2?}", self.start.elapsed());
        info!("successes: {}", self.successes);
        info!("failures: {}", self.failures);
        info!("gigabytes: {:.4}", self.gb);
    }
}
