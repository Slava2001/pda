use crate::{display::Display, key_event::KeyEventType, menu::{draw_line, entries::{FocusController, MenuEntry}}};

pub struct Label {
    text: String
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self { text: text.into() }
    }
}

impl MenuEntry for Label {
    fn update(&mut self, _parent: &mut dyn FocusController, _key_event: KeyEventType) {
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&self.text, display, x, y);
    }

    fn render(&self, _display: &mut Display, _x: i32, _y: i32) {
    }
}
