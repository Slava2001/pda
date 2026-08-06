use crate::{
    display::Display, key_event::{Key, KeyEventType}, menu::{
        draw_line,
        entries::{FocusController, MenuEntry},
    },
};

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

impl<T: std::fmt::Display + Send + Sync> MenuEntry for Select<T> {
    fn update(&mut self, _parent: &mut dyn FocusController, key_event: KeyEventType) {
        match key_event {
            KeyEventType::Press(key) | KeyEventType::Repeat(key) => match key {
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

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&format!("{}: {}", self.name, self.values[self.value]), display, x, y);
    }

    fn render(&self, _display: &mut Display, _x: i32, _y: i32) {}
}
