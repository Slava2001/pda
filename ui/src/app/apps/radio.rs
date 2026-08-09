use std::{future::pending, sync::Arc};

use crate::{
    app::{self, AppError}, display::Display, key_event::{Key, KeyEventType}, menu::{
        Menu, draw_line, entries::{FocusController, MenuEntry, button::Button, submenu::SubMenu, value::Value},
    }, si4732::{Si4732, cmd::Func},
};
use async_trait::async_trait;
use linux_embedded_hal::{
    Delay, I2cdev,
    gpio_cdev::{Chip, LineRequestFlags},
};
use tokio::sync::Mutex;

pub struct Radio {
    menu: Mutex<Menu>,
}

impl Radio {
    pub fn new() -> Self {
        Self {
            menu: Mutex::new(Menu::new()),
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
        let radio = Si4732::new(i2c, Delay, reset, 0x63);
        let radio = Arc::new(Mutex::new(radio));
        let mut menu = Menu::new();
        menu = menu.add(GetInfoMenuEntry::new(radio.clone()));
        let mut sub_menu = SubMenu::new("FM mode");
        let radio_c = radio.clone();
        sub_menu = sub_menu.add(Button::new("Init", move || {
            radio_c.try_lock().unwrap().init(Func::FmReceive).unwrap();
        }));
        let radio_c = radio.clone();
        sub_menu = sub_menu.add(Value::new("Volume", 63, 0, 63, 1, move |val| {
            radio_c.try_lock().unwrap().set_volume(val).unwrap();
        }));
        let radio_c = radio.clone();
        sub_menu = sub_menu.add(Value::new("Frequency", 64.0, 64.0, 108.0, 0.1, move |val| {
            radio_c.try_lock().unwrap().set_frequency((val * 1000.0) as u32).unwrap();
        }));
        let radio_c = radio.clone();
        sub_menu = sub_menu.add(Button::new("Read status", move || {
            let (_rssi, _snr) = radio_c.try_lock().unwrap().get_rssi_and_snr().unwrap_or((0, 0));
            // format!("RSSI: {rssi} SNR: {snr}")
        }));

        menu = menu.add(sub_menu);
        *self.menu.lock().await = menu;
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
