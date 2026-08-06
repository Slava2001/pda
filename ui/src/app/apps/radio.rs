use std::{future::pending, sync::Arc};

use crate::{
    app::{self, AppError},
    display::Display,
    key_event::{Key, KeyEventType},
    menu::{
        Menu, draw_line,
        entries::{FocusController, MenuEntry},
    },
    si4732::Si4732,
};
use async_trait::async_trait;
use linux_embedded_hal::{
    Delay, I2cdev,
    gpio_cdev::{Chip, LineRequestFlags},
};
use tokio::sync::Mutex;

pub struct Radio {
    radio: Arc<Mutex<Si4732<I2cdev, Delay>>>,
    menu: Mutex<Menu>,
}

impl Radio {
    pub fn new() -> Self {
        let i2c = I2cdev::new("/dev/i2c-0").unwrap();
        let mut chip = Chip::new("/dev/gpiochip0").unwrap();
        let reset = chip
            .get_line(0)
            .unwrap()
            .request(LineRequestFlags::OUTPUT, 1, "si4732-reset")
            .unwrap();
        let radio = Si4732::new(i2c, Delay, reset, 0x63);
        let radio = Arc::new(Mutex::new(radio));
        Self {
            menu: Mutex::new(Menu::new().add(GetInfoMenuEntry::new(radio.clone()))),
            radio,
        }
    }
}

#[async_trait]
impl app::App for Radio {
    async fn run(&self) -> Result<(), AppError> {
        pending().await
    }

    async fn key_event(&self, event: KeyEventType) -> Result<(), AppError> {
        self.menu.lock().await.update(event);
        Ok(())
    }

    async fn render(&self, display: &mut Display) -> Result<(), AppError> {
        self.menu.lock().await.render(display);
        Ok(())
    }
}

struct GetInfoMenuEntry {
    radio: Arc<Mutex<Si4732<I2cdev, Delay>>>,
    info: String,
}

impl GetInfoMenuEntry {
    fn new(radio: Arc<Mutex<Si4732<I2cdev, Delay>>>) -> Self {
        Self {
            radio,
            info: String::new(),
        }
    }
}

impl MenuEntry for GetInfoMenuEntry {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType) {
        if let KeyEventType::Press(Key::Enter) = key_event {
            if parent.is_focused() {
                parent.release_focus();
            } else {
                parent.grab_focus();
                let mut radio = self.radio.try_lock().unwrap();
                self.info = format!(
                    "{}\n{}\n{}{}\n{}{}\n{}{}\n{}{}",
                    "->Back",
                    "  Info:",
                    "    Part number:   ",
                    radio.get_part_number().unwrap(),
                    "    Library id:    ",
                    radio.get_library_id().unwrap(),
                    "    Firmware Ver:  ",
                    radio.get_firmware_ver().unwrap(),
                    "    Chip Revision: ",
                    radio.get_chip_rev().unwrap()
                );
            }
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line("Get chip info", display, x, y);
    }

    fn render(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&self.info, display, x, y + 20);
    }
}
