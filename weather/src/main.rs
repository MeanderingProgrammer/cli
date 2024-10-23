use anyhow::Result;
use weather::{forecast::WeatherClient, graph, info::InfoClient};

#[tokio::main]
async fn main() -> Result<()> {
    let info = InfoClient::default();

    let user_agent = info.user_agent()?;
    let weather = WeatherClient::new(&user_agent)?;

    let location = info.location().await?;
    let endpoint = weather.get_endpoint(&location).await?;
    let forecast = weather.get_forecast(&endpoint).await?;

    graph::create(&location.city, &forecast);
    Ok(())
}
