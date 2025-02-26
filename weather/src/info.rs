use anyhow::Result;
use serde::{Deserialize, de::DeserializeOwned};
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
struct Ip {
    origin: String,
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
    pub async fn location(&self) -> Result<Location> {
        let ip: Ip = self.get("http://httpbin.org/ip").await?;
        let endpoint = format!("http://ip-api.com/json/{}", &ip.origin);
        self.get(&endpoint).await
    }

    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        Ok(reqwest::get(endpoint).await?.json::<T>().await?)
    }

    pub fn user_agent(&self) -> Result<String> {
        let git_result = Command::new("git")
            .args(["config", "user.email"])
            .output()?;
        let mut git_email = String::from_utf8(git_result.stdout)?;
        git_email.pop();
        Ok(git_email)
    }
}
