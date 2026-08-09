use std::ops::{Add, Sub};

use crate::{display::Display, key_event::{Key, KeyEventType}, menu::{draw_line, entries::{FocusController, MenuEntry}}};

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

impl<T: std::fmt::Display + Send + Sync + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy, C: FnMut(T) -> () + Send + Sync> MenuEntry for Value<T, C> {
    fn update(&mut self, _parent: &mut dyn FocusController, key_event: KeyEventType) {
        match key_event {
            KeyEventType::Press(key) | KeyEventType::Repeat(key) => {
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

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&format!("{}: {:.2}", self.name, self.value), display, x, y);
    }

    fn render(&self, _display: &mut Display, _x: i32, _y: i32) {
    }
}
