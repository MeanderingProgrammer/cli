use crate::forecast::Forecast;
use chrono::{naive::Days, DateTime, Local};
use plotly::{
    common::{color::NamedColor, Marker, Mode, Title, Visible},
    layout::{Axis, RangeSlider},
    Layout, Plot, Scatter,
};

pub fn create(city: &str, forecast: &Forecast) {
    let days = forecast.map(|period| period.start_time);

    let mut plot = Plot::new();

    let precipitations = forecast.map(|period| period.probability_of_precipitation.value);
    let descriptions = forecast.map(|period| period.short_forecast.clone());
    let colors = forecast.map(|period| color(&period.short_forecast));
    plot.add_trace(
        Scatter::new(days.clone(), precipitations)
            .name("Rain")
            .text_array(descriptions)
            .marker(Marker::new().size(12).color_array(colors))
            .mode(Mode::LinesMarkers),
    );

    let temperatures = forecast.map(|period| period.temperature);
    plot.add_trace(
        Scatter::new(days.clone(), temperatures)
            .name("Temperature")
            .visible(Visible::LegendOnly),
    );

    let start = *days.first().unwrap();
    plot.set_layout(
        Layout::new()
            .height(800)
            .title(Title::with_text(format!("Weather in {city}")))
            .x_axis(
                Axis::new()
                    .title(Title::with_text("Date"))
                    .range_slider(RangeSlider::new().visible(true))
                    .range(vec![format_date(start), format_date(start + Days::new(1))]),
            )
            .y_axis(Axis::new().range(vec![0, 100])),
    );

    plot.show();
}

fn color(description: &str) -> NamedColor {
    // Colors: https://www.w3schools.com/cssref/css_colors.php
    match description {
        "Sunny" => NamedColor::Gold,
        "Mostly Sunny" => NamedColor::Gold,
        "Partly Sunny" => NamedColor::Gold,

        "Clear" => NamedColor::DarkBlue,
        "Mostly Clear" => NamedColor::DarkBlue,

        "Cloudy" => NamedColor::Blue,
        "Mostly Cloudy" => NamedColor::Blue,
        "Partly Cloudy" => NamedColor::Blue,

        "Patchy Fog" => NamedColor::LightGray,

        "Light Rain" => NamedColor::DarkGray,
        "Light Rain Likely" => NamedColor::DarkGray,
        "Chance Light Rain" => NamedColor::DarkGray,
        "Slight Chance Light Rain" => NamedColor::DarkGray,

        "Rain Showers" => NamedColor::Red,
        "Rain Showers Likely" => NamedColor::Red,
        "Chance Rain Showers" => NamedColor::Red,
        "Slight Chance Rain Showers" => NamedColor::Red,
        "Scattered Rain Showers" => NamedColor::Red,

        "Rain" => NamedColor::Red,
        "Rain Likely" => NamedColor::Red,

        "Chance Rain And Snow" => NamedColor::Red,
        "Slight Chance Rain And Snow" => NamedColor::Red,

        "Showers And Thunderstorms" => NamedColor::Red,
        "Showers And Thunderstorms Likely" => NamedColor::Red,
        "Slight Chance Showers And Thunderstorms" => NamedColor::Red,

        _ => panic!("No color matches: {description}"),
    }
}

fn format_date(date: DateTime<Local>) -> String {
    date.format("%Y-%m-%d %H:%M:%S").to_string()
}
