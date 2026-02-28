use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};
use shellexpand::env_with_context_no_errors as expand;
use toml::{Table, Value};

type Env = Vec<(String, String)>;
type Current = HashMap<String, String>;

#[derive(Debug)]
pub struct Resolver {
    files: Vec<PathBuf>,
}

impl Resolver {
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self { files }
    }

    pub fn get(&self) -> Result<Env> {
        let mut result = Env::default();
        let mut current = Current::default();
        for file in &self.files {
            let text = fs::read_to_string(file)?;
            let toml = Toml::new(&text)?;
            let env = self.expand(&mut current, toml.env())?;
            result.extend(env);
        }
        Ok(result)
    }

    fn expand(&self, current: &mut Current, env: Env) -> Result<Env> {
        let mut result = Env::default();
        for (key, value) in env {
            if current.contains_key(&key) {
                bail!("duplicate environment variable: {key}");
            }
            let value = expand(&value, |s| Some(Self::lookup(s, current))).to_string();
            result.push((key.clone(), value.clone()));
            current.insert(key, value);
        }
        Ok(result)
    }

    fn lookup(s: &str, current: &Current) -> String {
        current
            .get(s)
            .cloned()
            .or_else(|| std::env::var(s).ok())
            .unwrap_or_default()
    }
}

type Data = Vec<(Vec<String>, String)>;

#[derive(Debug)]
struct Toml {
    data: Data,
}

impl Toml {
    fn new(text: &str) -> Result<Self> {
        let table: Table = text.parse()?;
        let value = Value::Table(table);
        Ok(Self {
            data: Self::flatten(&value, Vec::default())?,
        })
    }

    fn flatten(value: &Value, path: Vec<String>) -> Result<Data> {
        match value {
            Value::Array(_) => bail!("toml arrays are not supported"),
            Value::String(v) => Ok([(path, v.to_string())].into()),
            Value::Integer(v) => Ok([(path, v.to_string())].into()),
            Value::Float(v) => Ok([(path, v.to_string())].into()),
            Value::Boolean(v) => Ok([(path, v.to_string())].into()),
            Value::Datetime(v) => Ok([(path, v.to_string())].into()),
            Value::Table(v) => {
                let mut result = Data::default();
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
        for (key, value) in &self.data {
            let key: Vec<_> = key.iter().map(|k| k.to_uppercase()).collect();
            result.push((key.join("_"), value.to_string()));
        }
        result
    }
}
