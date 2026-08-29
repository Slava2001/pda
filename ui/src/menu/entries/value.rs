use std::format;
use std::ops::{Add, Sub};

use async_trait::async_trait;
use crate::menu::entries::MenuEntry;
use crate::modules::display::DisplayIf;
use crate::modules::keyboard::Key;
use crate::{menu::entries::FocusController, modules::keyboard::KeyEvent};

pub struct Value<T: std::fmt::Display + Send + Sync, C: FnMut(T) -> ()> {
    name: String,
    value: T,
    min: T,
    max: T,
    step: T,
    cb: C
}

impl<T: std::fmt::Display + Send + Sync, C: FnMut(T) -> ()> Value<T, C> {
    pub fn new(name: &str, value: T, min: T, max: T, step: T, cb: C) -> Self {
        Self { name: name.into(), value, min, max, step, cb }
    }
}

#[async_trait]
impl<T: std::fmt::Display + Send + Sync + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy, C: FnMut(T) -> () + Send + Sync> MenuEntry for Value<T, C> {
    async fn update(&mut self, _parent: &mut dyn FocusController, key_event: KeyEvent) {
        match key_event {
            KeyEvent::Press(key) | KeyEvent::Repeat(key) => {
                match key {
                    Key::Left if self.value >= self.min + self.step => {
                        self.value = self.value - self.step;
                        (self.cb)(self.value);
                    }
                    Key::Right if self.value <= self.max - self.step => {
                        self.value = self.value + self.step;
                        (self.cb)(self.value);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    async fn render_line(&self) -> String {
        format!("{}: {:.2}", self.name, self.value)
    }

    async fn render(&self, _display: &mut DisplayIf) {
    }
}
