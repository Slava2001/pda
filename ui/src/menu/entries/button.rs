use crate::{display::Display, key_event::{Key, KeyEventType}, menu::{draw_line, entries::{FocusController, MenuEntry}}};

pub struct Button<T: FnMut() -> ()> {
    text: String,
    cb: T
}

impl<T: FnMut() -> ()> Button<T> {
    pub fn new(text: &str, cb: T) -> Self {
        Self { text: text.into(), cb }
    }
}

impl<T: FnMut() -> () + Send + Sync> MenuEntry for Button<T> {
    fn update(&mut self, _parent: &mut dyn FocusController, key_event: KeyEventType) {
        if let KeyEventType::Press(Key::Enter) = key_event {
            (self.cb)();
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&self.text, display, x, y);
    }

    fn render(&self, _display: &mut Display, _x: i32, _y: i32) {
    }
}
