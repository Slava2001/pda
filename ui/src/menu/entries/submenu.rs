use async_trait::async_trait;
use crate::menu::Menu;
use crate::menu::entries::MenuEntry;
use crate::menu::entries::label::Label;
use crate::modules::display::DisplayIf;
use crate::modules::keyboard::Key;
use crate::{menu::entries::FocusController, modules::keyboard::KeyEvent};

pub struct SubMenu {
    name: String,
    base: Menu
}

impl SubMenu {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), base: Menu::new().add(Label::new("back")) }
    }

    pub fn add<T: MenuEntry + 'static>(mut self, entry: T) -> Self {
        self.base = self.base.add(entry);
        self
    }
}

#[async_trait]
impl MenuEntry for SubMenu {
    async fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEvent) {
        if parent.is_focused() {
            if let KeyEvent::Press(Key::Enter) = key_event {
                if self.base.cursor == 0 {
                    parent.release_focus();
                    return;
                }
            }
            self.base.update(key_event).await;
        } else {
            if let KeyEvent::Press(Key::Enter) = key_event {
                parent.grab_focus();
            }
        }
    }

    async fn render_line(&self) -> String {
        self.name.clone()
    }

    async fn render(&self, display: &mut DisplayIf) {
        self.base.render(display).await;
    }
}
