use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

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
        println!("reading files from: {:?}", root);
        let files = self.read(&root)?;
        Ok(Directory::new(root, files))
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

#[derive(Debug)]
pub struct Directory {
    root: PathBuf,
    files: HashSet<PathBuf>,
}

impl Directory {
    pub fn new(root: PathBuf, files: Vec<PathBuf>) -> Self {
        let files = files
            .into_iter()
            .map(|file| file.strip_prefix(&root).unwrap().to_path_buf())
            .collect();
        Self { root, files }
    }

    pub fn missing(&self, src: &Self) -> Vec<PathBuf> {
        src.files
            .iter()
            .filter(|file| !self.files.contains(*file))
            .cloned()
            .collect()
    }

    pub fn copy(&self, src: &Self, file: &Path) -> Result<()> {
        let from = src.absolute(file);
        let to = self.absolute(file);

        let parent = to.parent().unwrap();
        if !parent.exists() {
            println!("creating missing directory: {:?}", parent);
            fs::create_dir_all(parent)?;
        }

        println!("copying: {:?} -> {:?}", from, to);
        fs::copy(&from, &to)?;

        Ok(())
    }

    fn absolute(&self, file: &Path) -> PathBuf {
        self.root.join(file)
    }
}
