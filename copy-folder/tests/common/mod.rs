use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use rand::{Rng, distr::Alphanumeric};
use tempfile::{TempDir, tempdir};

#[derive(Debug)]
pub struct TestData {
    root: TempDir,
    data: HashMap<String, String>,
}

#[allow(dead_code)]
impl TestData {
    pub fn new(files: &[&str]) -> Self {
        let root = tempdir().unwrap();
        assert!(root.path().is_dir());
        // generate some random text for each file
        let data: HashMap<_, _> = files
            .iter()
            .map(|file| (file.to_string(), Self::random_string(100)))
            .collect();
        // create the files and directories in root
        for (name, text) in data.iter() {
            let path = root.path().join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut file = File::create(&path).unwrap();
            write!(file, "{text}").unwrap();
        }
        Self { root, data }
    }

    fn random_string(n: usize) -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(n)
            .map(char::from)
            .collect()
    }

    pub fn path(&self) -> &Path {
        self.root.path()
    }

    pub fn initial(&self) -> HashMap<String, String> {
        self.data.clone()
    }

    pub fn current(&self) -> HashMap<String, String> {
        self.read(self.path())
    }

    fn read(&self, dir: &Path) -> HashMap<String, String> {
        assert!(dir.is_dir());
        let mut result = HashMap::default();
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let text = fs::read_to_string(&path).unwrap();
                let path = path.strip_prefix(self.path()).unwrap().to_str().unwrap();
                result.insert(path.into(), text);
            } else if path.is_dir() {
                result.extend(self.read(&path));
            } else {
                panic!("unhandled entry: {:?}", path);
            }
        }
        result
    }
}
