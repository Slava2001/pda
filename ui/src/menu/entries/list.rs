use crate::{
    display::Display, key_event::{Key, KeyEventType}, menu::{
        draw_line,
        entries::{FocusController, MenuEntry},
    },
};

pub struct List<T: AsRef<str>, V: std::fmt::Display + Send + Sync> {
    text: T,
    entries: Vec<V>,
    cursor: usize,
    is_selected: bool
}

impl<T: AsRef<str>, V: std::fmt::Display + Send + Sync> List<T, V> {
    pub fn new(text: T, entries: Vec<V>) -> Self {
        Self {
            text: text.into(),
            entries,
            cursor: 0,
            is_selected: false
        }
    }

    pub fn set_entries(&mut self, entries: Vec<V>) {
        self.entries = entries;
        self.cursor = 0;
    }

    pub fn value(&self) -> Option<&V> {
        if self.is_selected {
            Some(&self.entries[self.cursor])
        } else {
            None
        }
    }
}

impl<T: AsRef<str> + Send + Sync, V: std::fmt::Display + Send + Sync> MenuEntry for List<T, V> {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType) {
        if parent.is_focused() {
            match key_event {
                KeyEventType::Press(key) | KeyEventType::Repeat(key) => match key {
                    Key::Up if self.cursor > 0 => {
                        self.cursor = self.cursor - 1;
                    }
                    Key::Down if self.cursor + 1 < self.entries.len() => {
                        self.cursor = self.cursor + 1;
                    }
                    Key::Enter => {
                        self.is_selected = true;
                        parent.release_focus();
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            if let KeyEventType::Press(Key::Enter) = key_event {
                parent.grab_focus();
                self.cursor = 0;
            }
            self.is_selected = false;
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line(self.text.as_ref(), display, x, y);
    }

    fn render(&self, display: &mut Display, x: i32, y: i32) {
        for i in 0..self.entries.len() {
            if i == self.cursor {
                draw_line("->", display, x, y + 20 * (i as i32 + 1));
            }
            let str = &format!("{}", self.entries[i]);
            draw_line(str, display, x + 20, y + 20 * (i as i32 + 1));
        }
    }
}
