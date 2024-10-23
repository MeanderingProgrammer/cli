use crate::info::Location;
use anyhow::Result;
use chrono::{DateTime, Local};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EndpointsProperties {
    forecast_hourly: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Endpoints {
    properties: EndpointsProperties,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastPrecipitation {
    pub value: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForecastPeriod {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub temperature: u32,
    pub probability_of_precipitation: ForecastPrecipitation,
    pub short_forecast: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ForecastProperties {
    periods: Vec<ForecastPeriod>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Forecast {
    properties: ForecastProperties,
}

impl Forecast {
    pub fn map<T>(&self, f: fn(&ForecastPeriod) -> T) -> Vec<T> {
        self.properties.periods.iter().map(f).collect()
    }
}

#[derive(Debug)]
pub struct WeatherClient {
    client: Client,
}

impl WeatherClient {
    pub fn new(user_agent: &str) -> Result<Self> {
        let client = Client::builder().user_agent(user_agent).build()?;
        let weather_client = Self { client };
        Ok(weather_client)
    }

    pub async fn get_endpoint(&self, location: &Location) -> Result<String> {
        let (lat, lon) = (location.lat, location.lon);
        let endpoint = format!("https://api.weather.gov/points/{lat},{lon}");
        let endpoints: Endpoints = self.get(&endpoint).await?;
        Ok(endpoints.properties.forecast_hourly)
    }

    pub async fn get_forecast(&self, endpoint: &str) -> Result<Forecast> {
        self.get(endpoint).await
    }

    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        Ok(self.client.get(endpoint).send().await?.json::<T>().await?)
    }
}
