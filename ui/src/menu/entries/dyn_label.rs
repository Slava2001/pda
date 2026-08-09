use crate::{display::Display, key_event::KeyEventType, menu::{draw_line, entries::{FocusController, MenuEntry}}};

pub struct DynLabel<T: Fn() -> String> {
    cb: T
}

impl<T: Fn() -> String> DynLabel<T> {
    pub fn new(cb: T) -> Self {
        Self { cb }
    }
}

impl<T: Fn() -> String + Send + Sync> MenuEntry for DynLabel<T> {
    fn update(&mut self, _parent: &mut dyn FocusController, _key_event: KeyEventType) {
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&(self.cb)(), display, x, y);
    }

    fn render(&self, _display: &mut Display, _x: i32, _y: i32) {
    }
}
