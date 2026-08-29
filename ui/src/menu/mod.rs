pub mod entries;
use crate::{
    menu::entries::{FocusController, MenuEntry},
    modules::{
        display::DisplayIf,
        keyboard::{Key, KeyEvent},
    },
};

pub struct Menu {
    enters: Vec<Box<dyn MenuEntry>>,
    cursor: usize,
    is_focused: bool,
    offset: usize,
    window: usize,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            enters: Vec::new(),
            cursor: 0,
            is_focused: false,
            offset: 0,
            window: 1,
        }
    }

    pub fn set_window(&mut self, w: usize) {
        self.window = w.min(self.enters.len());
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn reset_state(&mut self) {
        self.cursor = 0;
        self.offset = 0;
        self.is_focused = false;
    }

    pub fn add<T: MenuEntry + 'static>(mut self, entry: T) -> Self {
        self.enters.push(Box::new(entry) as Box<dyn MenuEntry>);
        self
    }

    pub async fn update(&mut self, key_event: KeyEvent) {
        if self.enters.is_empty() {
            return;
        }
        if !self.is_focused {
            match key_event {
                KeyEvent::Press(key) | KeyEvent::Repeat(key) => match key {
                    Key::Up if self.cursor > 0 => {
                        self.cursor = self.cursor - 1;
                        if self.cursor < self.offset {
                            self.offset = self.cursor;
                        }
                    }
                    Key::Down if self.cursor + 1 < self.enters.len() => {
                        self.cursor = self.cursor + 1;
                        if self.cursor > self.offset + (self.window - 1) {
                            self.offset = self.cursor - (self.window - 1);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        let mut focus_cntl = MenuFocusController(&mut self.is_focused);
        self.enters[self.cursor]
            .update(&mut focus_cntl as &mut dyn FocusController, key_event)
            .await;
    }

    pub async fn render(&self, display: &mut DisplayIf) {
        if self.is_focused {
            self.enters[self.cursor].render(display).await;
        } else {
            for (i, e) in self.enters[self.offset..self.offset + self.window]
                .iter()
                .enumerate()
            {
                let cursor = if i + self.offset == self.cursor {
                    "->"
                } else {
                    "  "
                };
                display
                    .draw_line(format!("{}{}", cursor, e.render_line().await), i)
                    .await
                    .ok();
            }
        }
    }
}

struct MenuFocusController<'a>(&'a mut bool);

impl<'a> FocusController for MenuFocusController<'a> {
    fn grab_focus(&mut self) {
        *self.0 = true;
    }

    fn release_focus(&mut self) {
        *self.0 = false;
    }

    fn is_focused(&mut self) -> bool {
        *self.0
    }
}
