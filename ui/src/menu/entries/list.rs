use std::format;

use crate::menu::Menu;
use crate::menu::entries::MenuEntry;
use crate::menu::entries::label::Label;
use crate::modules::display::DisplayIf;
use crate::modules::keyboard::Key;
use crate::{menu::entries::FocusController, modules::keyboard::KeyEvent};
use async_trait::async_trait;

pub struct List<CB: Send + Sync + FnMut(&V) -> (), T: AsRef<str>, V: std::fmt::Display + Send + Sync> {
    text: T,
    entries: Vec<V>,
    cb: CB,
    base: Menu
}

impl<CB: Send + Sync + FnMut(&V) -> (), T: AsRef<str>, V: std::fmt::Display + Send + Sync> List<CB, T, V> {
    pub fn new(text: T, entries: Vec<V>, cb: CB) -> Self {
        let mut list = Self {
            text,
            entries: Vec::new(),
            cb,
            base: Menu::new()
        };
        list.set_entries(entries);
        list
    }

    pub fn set_window(&mut self, w: usize) {
        self.base.set_window(w);
    }

    pub fn set_entries(&mut self, entries: Vec<V>) {
        self.entries = entries;
        let mut base = Menu::new();
        for e in &self.entries {
            base = base.add(Label::new(&format!("{e}")));
        }
        self.base = base;
    }
}

#[async_trait]
impl<CB: Send + Sync + FnMut(&V) -> (), T: AsRef<str> + Send + Sync, V: std::fmt::Display + Send + Sync> MenuEntry for List<CB, T, V> {
    async fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEvent) {
        if parent.is_focused() {
            self.base.update(key_event).await;
            if let KeyEvent::Press(Key::Enter) = key_event {
                (self.cb)(&self.entries[self.base.cursor()]);
                self.base.reset_state();
            }
        } else {
            if let KeyEvent::Press(Key::Enter) = key_event {
                parent.grab_focus();
            }
        }
    }

    async fn render_line(&self) -> String {
        self.text.as_ref().to_string()
    }

    async fn render(&self, display: &mut DisplayIf) {
        self.base.render(display).await
    }
}
