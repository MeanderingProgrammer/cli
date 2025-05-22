use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use log::info;

#[derive(Debug)]
pub struct Reader {
    names: HashSet<String>,
    prefixes: Vec<String>,
}

impl Reader {
    pub fn new<I>(names: I, prefixes: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self {
            names: names.into_iter().collect(),
            prefixes: prefixes.into_iter().collect(),
        }
    }

    pub fn get(&self, root: &Path) -> Result<HashSet<PathBuf>> {
        info!("reading from: {:?}", root);
        let files = self.read(root, root)?;
        info!("number of files: {}", files.len());
        Ok(files)
    }

    fn read(&self, root: &Path, dir: &Path) -> Result<HashSet<PathBuf>> {
        if !dir.is_dir() {
            bail!("not a directory: {:?}", dir)
        }
        let mut result = HashSet::default();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() {
                if !self.skip(&path) {
                    let path = path.strip_prefix(root).unwrap().to_path_buf();
                    result.insert(path);
                }
            } else if path.is_dir() {
                result.extend(self.read(root, &path)?);
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
