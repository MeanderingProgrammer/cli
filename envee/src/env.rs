use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};
use toml::Value;

#[derive(Debug)]
pub struct Resolver {
    files: Vec<PathBuf>,
}

impl Resolver {
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self { files }
    }

    pub fn get(&self) -> Result<HashMap<String, String>> {
        let mut result = HashMap::default();
        for file in &self.files {
            let text = fs::read_to_string(file)?;
            let value: Value = text.parse()?;
            let env = Env::new(&value)?;
            result.extend(env.normalize());
        }
        Ok(result)
    }
}

#[derive(Debug)]
struct Env {
    table: HashMap<Vec<String>, String>,
}

impl Env {
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

    fn normalize(&self) -> HashMap<String, String> {
        let mut result = HashMap::default();
        for (key, value) in &self.table {
            let key: Vec<_> = key.iter().map(|k| k.to_uppercase()).collect();
            result.insert(key.join("_"), value.to_string());
        }
        result
    }
}
