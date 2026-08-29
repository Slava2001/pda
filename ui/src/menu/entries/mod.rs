use crate::modules::{display::DisplayIf, keyboard::KeyEvent};
use async_trait::async_trait;

pub mod button;
pub mod dyn_label;
pub mod label;
pub mod list;
pub mod select;
pub mod submenu;
pub mod value;

pub trait FocusController: Sync + Send {
    fn grab_focus(&mut self);
    fn release_focus(&mut self);
    fn is_focused(&mut self) -> bool;
}

pub struct AlwaysFocused;
impl FocusController for AlwaysFocused {
    fn grab_focus(&mut self) {}
    fn release_focus(&mut self) {}
    fn is_focused(&mut self) -> bool {
        true
    }
}

#[async_trait]
pub trait MenuEntry: Send + Sync {
    async fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEvent);
    async fn render_line(&self) -> String;
    async fn render(&self, display: &mut DisplayIf);
}
