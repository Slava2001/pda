use async_trait::async_trait;
use crate::menu::entries::MenuEntry;
use crate::modules::display::DisplayIf;
use crate::{menu::entries::FocusController, modules::keyboard::KeyEvent};

pub struct DynLabel<T: Fn() -> String> {
    cb: T
}

impl<T: Fn() -> String> DynLabel<T> {
    pub fn new(cb: T) -> Self {
        Self { cb }
    }
}

#[async_trait]
impl<T: Fn() -> String + Send + Sync> MenuEntry for DynLabel<T> {
    async fn update(&mut self, _parent: &mut dyn FocusController, _key_event: KeyEvent) {
    }

    async fn render_line(&self) -> String {
        (self.cb)()
    }

    async fn render(&self, _display: &mut DisplayIf) {
    }
}
