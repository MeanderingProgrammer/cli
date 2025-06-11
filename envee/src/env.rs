use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};
use toml::Value;

#[derive(Debug)]
pub struct Resolver {
    files: Vec<PathBuf>,
    depth: usize,
}

impl Resolver {
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self { files, depth: 20 }
    }

    pub fn get(&self) -> Result<Env> {
        let mut result = HashMap::default();
        for file in &self.files {
            let text = fs::read_to_string(file)?;
            let value: Value = text.parse()?;
            let env = Toml::new(&value)?.env();
            result.extend(self.expand(env)?);
        }
        Ok(result)
    }

    fn expand(&self, env: Env) -> Result<Env> {
        let mut result = env;
        for _ in 0..self.depth {
            let next = Self::expand_once(&result);
            if next == result {
                return Ok(result);
            }
            result = next;
        }
        bail!("failed to expand variables after {} iterations", self.depth);
    }

    fn expand_once(env: &Env) -> Env {
        env.iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    match shellexpand::env_with_context(value, |s| Self::lookup(s, env)) {
                        Ok(value) => value.to_string(),
                        Err(_) => value.to_string(),
                    },
                )
            })
            .collect()
    }

    fn lookup(s: &str, env: &Env) -> Result<Option<String>> {
        match env.get(s) {
            Some(value) => Ok(Some(value.clone())),
            None => Ok(std::env::var(s).map(Some)?),
        }
    }
}

type Env = HashMap<String, String>;

#[derive(Debug)]
struct Toml {
    table: HashMap<Vec<String>, String>,
}

impl Toml {
    fn new(value: &Value) -> Result<Self> {
        Ok(Self {
            table: Self::flatten(value, Vec::default())?,
        })
    }

    fn flatten(value: &Value, path: Vec<String>) -> Result<HashMap<Vec<String>, String>> {
        match value {
            Value::Array(_) => bail!("toml arrays are not supported"),
            Value::String(v) => Ok([(path, v.to_string())].into()),
            Value::Integer(v) => Ok([(path, v.to_string())].into()),
            Value::Float(v) => Ok([(path, v.to_string())].into()),
            Value::Boolean(v) => Ok([(path, v.to_string())].into()),
            Value::Datetime(v) => Ok([(path, v.to_string())].into()),
            Value::Table(v) => {
                let mut result = HashMap::default();
                for (key, value) in v {
                    let mut path = path.clone();
                    path.push(key.to_string());
                    result.extend(Self::flatten(value, path)?);
                }
                Ok(result)
            }
        }
    }

    fn env(&self) -> Env {
        let mut result = Env::default();
        for (key, value) in &self.table {
            let key: Vec<_> = key.iter().map(|k| k.to_uppercase()).collect();
            result.insert(key.join("_"), value.to_string());
        }
        result
    }
}
