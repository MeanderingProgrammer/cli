use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use log::info;

#[derive(Debug)]
pub struct Directory {
    pub root: PathBuf,
    pub files: HashSet<PathBuf>,
}

#[derive(Debug)]
pub struct Reader {
    names: HashSet<String>,
    prefixes: Vec<String>,
}

impl Reader {
    pub fn new(names: Vec<String>, prefixes: Vec<String>) -> Self {
        Self {
            names: HashSet::from_iter(names),
            prefixes,
        }
    }

    pub fn get(&self, root: PathBuf) -> Result<Directory> {
        info!("reading from: {:?}", root);
        let files: HashSet<PathBuf> = self
            .read(&root)?
            .into_iter()
            .map(|file| file.strip_prefix(&root).unwrap().to_path_buf())
            .collect();
        info!("number of files: {}", files.len());
        Ok(Directory { root, files })
    }

    fn read(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        if !dir.is_dir() {
            bail!("not a directory: {:?}", dir)
        }
        let mut result = Vec::default();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() {
                if !self.skip(&path) {
                    result.push(path);
                }
            } else if path.is_dir() {
                let mut nested = self.read(&path)?;
                result.append(&mut nested);
            } else {
                bail!("unhandled entry: {:?}", path);
            }
        }
        Ok(result)
    }

    fn skip(&self, path: &Path) -> bool {
        let name = path.file_name().unwrap().to_str().unwrap();
        if self.names.contains(name) {
            return true;
        }
        self.prefixes.iter().any(|prefix| name.starts_with(prefix))
    }
}
