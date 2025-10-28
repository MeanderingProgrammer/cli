use std::process::Command;

use anyhow::Result;
use log::info;
use serde::{Deserialize, de::DeserializeOwned};

#[derive(Debug, Clone, Deserialize)]
struct Address {
    ip: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    pub city: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Default)]
pub struct InfoClient {}

impl InfoClient {
    pub fn user_agent(&self) -> Result<String> {
        let git_result = Command::new("git")
            .args(["config", "user.email"])
            .output()?;
        let mut git_email = String::from_utf8(git_result.stdout)?;
        git_email.pop();
        Ok(git_email)
    }

    pub async fn location(&self) -> Result<Location> {
        let address: Address = self.get("https://api.ipify.org?format=json").await?;
        info!("{:?}", address);
        let endpoint = format!("http://ip-api.com/json/{}", &address.ip);
        self.get(&endpoint).await
    }

    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        Ok(reqwest::get(endpoint).await?.json::<T>().await?)
    }
}
