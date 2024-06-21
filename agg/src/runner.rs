use crate::{asciicast, events, fonts, renderer, vt, RendererName, ThemeName};
use anyhow::{bail, Ok, Result};
use gifski::progress::ProgressBar;
use std::fs::File;
use std::time::Instant;

#[derive(Debug)]
pub struct Runner {
    pub renderer: RendererName,
    pub fonts: Vec<String>,
    pub font_size: f32,
    pub line_height: f32,
    pub theme: ThemeName,
    pub speed: f64,
    pub fps_cap: u8,
    pub last_frame_duration: f64,
}

impl Runner {
    pub fn run(&self, input: File, output: File) -> Result<()> {
        let (header, events) = asciicast::open(input)?;

        let terminal_size = header.terminal_size;
        log::info!("terminal size: {}x{}", terminal_size.0, terminal_size.1);

        let stdout = std::iter::once(events::Event::default()).chain(events);
        let stdout = events::accelerate(stdout, self.speed);
        let stdout = events::batch(stdout, self.fps_cap);
        let stdout: Vec<events::Event> = stdout.collect();

        let count = stdout.len() as u64;
        let frames = vt::frames(stdout.into_iter(), terminal_size);

        let font_db = fonts::CachingFontDb::default();
        let font_families = font_db.available_fonts(&self.fonts);
        if font_families.is_empty() {
            bail!("no faces matching font families {:?}", self.fonts);
        }
        log::info!("selected font families: {:?}", font_families);

        log::info!("selected theme: {:?}", self.theme);

        let settings = renderer::Settings {
            terminal_size,
            font_db,
            font_families,
            font_size: self.font_size,
            line_height: self.line_height,
            theme: self.theme.clone().try_into()?,
        };

        let mut renderer = self.renderer.get_renderer(settings);

        let (width, height) = renderer.pixel_size();
        log::info!("gif dimensions: {}x{}", width, height);

        let settings = gifski::Settings {
            width: Some(width as u32),
            height: Some(height as u32),
            fast: true,
            repeat: gifski::Repeat::Infinite,
            ..Default::default()
        };

        let (collector, writer) = gifski::new(settings)?;
        let start_time = Instant::now();

        std::thread::scope(|s| {
            let writer_handle = s.spawn(move || {
                let mut pr = ProgressBar::new(count);
                let result = writer.write(output, &mut pr);
                pr.finish();
                println!();
                result
            });
            for (i, (time, frame)) in frames.enumerate() {
                let image = renderer.render(frame);
                let time = if i == 0 { 0.0 } else { time };
                collector.add_frame_rgba(i, image, time + self.last_frame_duration)?;
            }
            drop(collector);
            writer_handle.join().unwrap()?;
            Ok(())
        })?;

        let elapsed = start_time.elapsed().as_secs_f32();
        log::info!("rendering finished in {}s", elapsed);

        Ok(())
    }
}
