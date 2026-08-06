use std::future::pending;

use crate::{
    app::{self, AppError},
    display::Display,
    key_event::KeyEventType,
    menu::draw_line,
    si4732::Si4732,
};
use async_trait::async_trait;
use linux_embedded_hal::{
    Delay, I2cdev,
    gpio_cdev::{Chip, LineRequestFlags},
};
use tokio::sync::Mutex;

pub struct Radio {
    log: Mutex<String>,
}

impl Radio {
    pub fn new() -> Self {
        Self {
            log: Mutex::new("".into()),
        }
    }
}

#[async_trait]
impl app::App for Radio {
    async fn run(&self) -> Result<(), AppError> {
        let i2c = I2cdev::new("/dev/i2c-0").unwrap();
        let mut chip = Chip::new("/dev/gpiochip0").unwrap();
        let reset = chip
            .get_line(0)
            .unwrap()
            .request(LineRequestFlags::OUTPUT, 1, "si4732-reset")
            .unwrap();
        *self.log.lock().await = "Starting...".into();
        let mut radio = Si4732::new(i2c, Delay, reset, 0x63);

        *self.log.lock().await += &format!(
            "PN: {}\nLib id: {}\nFW Ver: {}\nChip Rev: {}\n",
            radio.get_part_number().unwrap(),
            radio.get_library_id().unwrap(),
            radio.get_firmware_ver().unwrap(),
            radio.get_chip_rev().unwrap()
        );

        radio.init().unwrap();
        *self.log.lock().await += "Started: OK";
        pending().await
    }

    async fn key_event(&self, _event: KeyEventType) -> Result<(), AppError> {
        Ok(())
    }

    async fn render(&self, display: &mut Display) -> Result<(), AppError> {
        draw_line(self.log.lock().await.as_str(), display, 10, 20);
        Ok(())
    }
}
