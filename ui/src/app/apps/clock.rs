use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use embedded_graphics::{Drawable, geometry::Point, mono_font::{MonoTextStyle, ascii::FONT_10X20}, pixelcolor::{Rgb565, RgbColor}, text::Text};
use tokio::sync::Notify;
use crate::{app::{self, AppError}, display::Display, key_event::{Key, KeyEventType}};

pub struct Clock {
    stop: Notify,
}

impl Clock {
    pub fn new() -> Self {
        Self { stop: Notify::new() }
    }
}

#[async_trait]
impl app::App for Clock {
    async fn run(&self) -> Result<(), AppError> {
        self.stop.notified().await;
        Ok(())
    }

    async fn key_event(&self, event: KeyEventType) -> Result<(), AppError> {
        if let KeyEventType::Press(Key::Left) = event {
            self.stop.notify_one();
        }
        Ok(())
    }
    async fn render(&self, display: &mut Display) -> Result<(), AppError> {
        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let text = format!("Time: {:02}:{:02}:{:02}", (now / 3600) % 24, (now / 60) % 60, now % 60);
        Text::new(&text, Point::new(10, 30), style).draw(display).ok();
        Ok(())
    }
}
