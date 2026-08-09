use crate::{display::Display, key_event::KeyEventType};

pub mod label;
pub mod value;
pub mod submenu;
pub mod select;
pub mod button;
pub mod dyn_label;
pub mod list;

pub trait FocusController {
    fn grab_focus(&mut self);
    fn release_focus(&mut self);
    fn is_focused(&mut self) -> bool;
}

pub trait MenuEntry: Send + Sync {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType);
    fn render_line(&self, display: &mut Display, x: i32, y: i32);
    fn render(&self, display: &mut Display, x: i32, y: i32);
}
