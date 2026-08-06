use std::ops::{Add, Sub};

use crate::{display::Display, key_event::{Key, KeyEventType}, menu::{draw_line, entries::{FocusController, MenuEntry}}};

pub struct Value<T: std::fmt::Display + Send + Sync> {
    name: String,
    value: T,
    min: T,
    max: T,
    step: T,
}

impl<T: std::fmt::Display + Send + Sync> Value<T> {
    pub fn new(name: &str, value: T, min: T, max: T, step: T) -> Self {
        Self { name: name.into(), value, min, max, step }
    }
}

impl<T: std::fmt::Display + Send + Sync + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy> MenuEntry for Value<T> {
    fn update(&mut self, _parent: &mut dyn FocusController, key_event: KeyEventType) {
        match key_event {
            KeyEventType::Press(key) | KeyEventType::Repeat(key) => {
                match key {
                    Key::Left if self.value >= self.min + self.step => {
                        self.value = self.value - self.step;
                    }
                    Key::Right if self.value <= self.max - self.step => {
                        self.value = self.value + self.step;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&format!("{}: {}", self.name, self.value), display, x, y);
    }

    fn render(&self, _display: &mut Display, _x: i32, _y: i32) {
    }
}
