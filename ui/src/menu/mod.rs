pub mod entries;

use crate::{
    display::Display,
    key_event::{Key, KeyEventType},
    menu::entries::{FocusController, MenuEntry},
};
use embedded_graphics::{
    Drawable,
    geometry::Point,
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::{Rgb565, RgbColor},
    text::Text,
};

pub struct Menu {
    enters: Vec<Box<dyn MenuEntry>>,
    cursor: usize,
    is_focused: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            enters: Vec::new(),
            cursor: 0,
            is_focused: false,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn add<T: MenuEntry + 'static>(mut self, entry: T) -> Self {
        self.enters.push(Box::new(entry) as Box<dyn MenuEntry>);
        self
    }

    pub fn update(&mut self, key_event: KeyEventType) {
        if self.enters.is_empty() {
            return;
        }
        if !self.is_focused {
            match key_event {
                KeyEventType::Press(key) | KeyEventType::Repeat(key) => match key {
                    Key::Up if self.cursor > 0 => {
                        self.cursor = self.cursor - 1;
                    }
                    Key::Down if self.cursor + 1 < self.enters.len() => {
                        self.cursor = self.cursor + 1;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        let mut focus_cntl = MenuFocusController(&mut self.is_focused);
        self.enters[self.cursor].update(&mut focus_cntl as &mut dyn FocusController, key_event);
    }

    pub fn render(&self, display: &mut Display) {
        if self.is_focused {
            self.enters[self.cursor].render(display, 0, 0);
        } else {
            for (i, e) in self.enters.iter().enumerate() {
                if i == self.cursor {
                    draw_line("->", display, 0, 20 * (self.cursor as i32 + 1));
                }
                e.render_line(display, 20, 20 * (i as i32 + 1));
            }
        }
    }
}

pub fn draw_line(text: &str, display: &mut Display, x: i32, y: i32) {
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::new(text, Point::new(x, y), style).draw(display).ok();
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
