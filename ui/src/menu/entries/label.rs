use async_trait::async_trait;
use crate::menu::entries::MenuEntry;
use crate::modules::display::DisplayIf;
use crate::{menu::entries::FocusController, modules::keyboard::KeyEvent};

pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self { text: text.into() }
    }
}

#[async_trait]
impl MenuEntry for Label {
    async fn update(&mut self, _parent: &mut dyn FocusController, _key_event: KeyEvent) {}

    async fn render_line(&self) -> String {
        self.text.clone()
    }

    async fn render(&self, _display: &mut DisplayIf) {}
}
