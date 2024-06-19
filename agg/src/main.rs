use agg::{Config, RendererName, ThemeName};
use anyhow::Result;
use clap::{ArgAction, Parser};
use std::fs::File;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// asciicast path/filename
    input_filename: String,

    /// GIF path/filename
    output_filename: String,

    /// Select frame rendering backend
    #[clap(long, value_enum, default_value_t = RendererName::default())]
    renderer: RendererName,

    /// Specify font families
    #[clap(long, default_values_t = [String::from("Monaspace Neon"), String::from("Symbols Nerd Font"), String::from("JetBrainsMono Nerd Font")])]
    font: Vec<String>,

    /// Specify font size (in pixels)
    #[clap(long, default_value_t = 14)]
    font_size: usize,

    /// Specify line height
    #[clap(long, default_value_t = 1.4)]
    line_height: f64,

    /// Select color theme
    #[clap(long, value_enum, default_value_t = ThemeName::default())]
    theme: ThemeName,

    /// Adjust playback speed
    #[clap(long, default_value_t = 1.0)]
    speed: f64,

    /// Set FPS cap
    #[clap(long, default_value_t = 30)]
    fps_cap: u8,

    /// Set last frame duration
    #[clap(long, default_value_t = 1.0)]
    last_frame_duration: f64,

    /// Enable verbose logging
    #[clap(short, long, action = ArgAction::Count)]
    verbose: u8,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "error",
        1 => "info",
        _ => "debug",
    };

    let env = env_logger::Env::default().default_filter_or(log_level);
    env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .init();

    let input = File::open(&cli.input_filename)?;
    let output = File::create(&cli.output_filename)?;
    let config = Config {
        renderer: cli.renderer,
        fonts: cli.font,
        font_size: cli.font_size,
        line_height: cli.line_height,
        theme: cli.theme,
        speed: cli.speed,
        fps_cap: cli.fps_cap,
        last_frame_duration: cli.last_frame_duration,
    };
    agg::run(input, output, config)
}
