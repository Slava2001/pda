use crate::{display::Display, key_event::{Key, KeyEventType}, menu::{Menu, draw_line, entries::{FocusController, MenuEntry, label::Label}}};

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

impl MenuEntry for SubMenu {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType) {
        if parent.is_focused() {
            if let KeyEventType::Press(Key::Enter) = key_event {
                if self.base.cursor == 0 {
                    parent.release_focus();
                    return;
                }
            }
            self.base.update(key_event);
        } else {
            if let KeyEventType::Press(Key::Enter) = key_event {
                parent.grab_focus();
            }
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(&self.name, display, x, y);
    }

    fn render(&self, display: &mut Display, _x: i32, _y: i32) {
        self.base.render(display);
    }
}
