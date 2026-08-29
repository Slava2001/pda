use async_trait::async_trait;
use crate::menu::entries::MenuEntry;
use crate::modules::display::DisplayIf;
use crate::modules::keyboard::Key;
use crate::{menu::entries::FocusController, modules::keyboard::KeyEvent};

pub struct Button<T: FnMut() -> ()> {
    text: String,
    cb: T
}

impl<T: FnMut() -> ()> Button<T> {
    pub fn new(text: &str, cb: T) -> Self {
        Self { text: text.into(), cb }
    }
}

#[async_trait]
impl<T: FnMut() -> () + Send + Sync> MenuEntry for Button<T> {
    async fn update(&mut self, _parent: &mut dyn FocusController, key_event: KeyEvent) {
        if let KeyEvent::Press(Key::Enter) = key_event {
            (self.cb)();
        }
    }

    async fn render_line(&self) -> String {
        self.text.clone()
    }

    async fn render(&self, _display: &mut DisplayIf) {
    }
}
