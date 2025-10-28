use anyhow::Result;
use env_logger::Builder;
use log::{LevelFilter, info};

use weather::{forecast::WeatherClient, graph, info::InfoClient};

#[tokio::main]
async fn main() -> Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();

    let info = InfoClient::default();

    let user_agent = info.user_agent()?;
    info!("UserAgent {:?}", user_agent);

    let location = info.location().await?;
    info!("{:?}", location);

    let weather = WeatherClient::new(&user_agent)?;
    let endpoint = weather.get_endpoint(&location).await?;
    let forecast = weather.get_forecast(&endpoint).await?;

    graph::create(&location.city, &forecast);
    Ok(())
}
