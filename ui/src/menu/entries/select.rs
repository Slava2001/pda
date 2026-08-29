use std::format;

use async_trait::async_trait;
use crate::menu::entries::MenuEntry;
use crate::modules::display::DisplayIf;
use crate::modules::keyboard::Key;
use crate::{menu::entries::FocusController, modules::keyboard::KeyEvent};

pub struct Select<T: std::fmt::Display + Send + Sync> {
    name: String,
    values: Vec<T>,
    value: usize,
}

impl<T: std::fmt::Display + Send + Sync> Select<T> {
    pub fn new(name: &str, values: Vec<T>, value: usize) -> Self {
        Self {
            name: name.into(),
            values,
            value,
        }
    }
}

#[async_trait]
impl<T: std::fmt::Display + Send + Sync> MenuEntry for Select<T> {
    async fn update(&mut self, _parent: &mut dyn FocusController, key_event: KeyEvent) {
        match key_event {
            KeyEvent::Press(key) | KeyEvent::Repeat(key) => match key {
                Key::Left if self.value > 0 => {
                    self.value = self.value - 1;
                }
                Key::Right if self.value + 1 < self.values.len() => {
                    self.value = self.value + 1;
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn render_line(&self) -> String {
        format!("{}: {}", self.name, self.values[self.value])
    }

    async fn render(&self, _display: &mut DisplayIf) {}
}
