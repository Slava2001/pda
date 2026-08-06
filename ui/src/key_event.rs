use evdev::{Device, EventStream, InputEventKind};
use std::{io, path::PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Up,
    Left,
    Down,
    Right,
    VolumeUp,
    VolumeDown,
    Enter,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum KeyEventType {
    Press(Key),
    Repeat(Key),
    Release(Key),
}

pub struct KeyEvent {
    stream: EventStream,
}

const KB_PATH: &str = "gpio_keys";

impl KeyEvent {
    pub fn new() -> io::Result<Self> {
        let pred = |(_, dev): &(PathBuf, Device)| dev.name() == Some(KB_PATH);
        let find_err = io::Error::new(io::ErrorKind::NotFound, "Device not found");
        let dev = evdev::enumerate().find(pred).ok_or(find_err)?.1;
        let stream = dev.into_event_stream()?;
        Ok(Self { stream })
    }

    pub async fn next(&mut self) -> KeyEventType {
        loop {
            let event = self.stream.next_event().await;
            if let Ok(event) = event {
                if let InputEventKind::Key(key) = event.kind() {
                    let Some(key) = (match key {
                        evdev::Key::KEY_UP => Some(Key::Up),
                        evdev::Key::KEY_LEFT => Some(Key::Left),
                        evdev::Key::KEY_DOWN => Some(Key::Down),
                        evdev::Key::KEY_RIGHT => Some(Key::Right),
                        evdev::Key::KEY_VOLUMEUP => Some(Key::VolumeUp),
                        evdev::Key::KEY_VOLUMEDOWN => Some(Key::VolumeDown),
                        evdev::Key::KEY_ENTER => Some(Key::Enter),
                        _ => None,
                    }) else {
                        continue;
                    };
                    match event.value() {
                        0 => break KeyEventType::Release(key),
                        1 => break KeyEventType::Press(key),
                        2 => break KeyEventType::Repeat(key),
                        _ => {}
                    }
                }
            }
        }
    }
}
