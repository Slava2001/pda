use std::sync::Arc;

use crate::{
    core::{interface::IfMngr, module::Module},
    create_imc_interface,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::select;

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

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Press(Key),
    Repeat(Key),
    Release(Key),
}

create_imc_interface! {
    pub interface KeyboardIf {
        fn subscribe() -> tokio::sync::broadcast::Receiver<KeyEvent>;
    }
}

pub struct Keyboard {}

impl Keyboard {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for Keyboard {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let tx = Arc::new(tokio::sync::broadcast::channel(10).0);
        let tx_c = tx.clone();
        let mut keyboard_if = KeyboardIfBackend::builder()
            .on_subscribe(move || {
                let rx = tx_c.subscribe();
                async move { rx }
            })
            .build()
            .context("Failed to build keyboard interface")?;
        let frontend = keyboard_if.get_if();
        if_mngr
            .reg("keyboard", move || frontend.clone())
            .await
            .context("Failed to reg keyboard interface")?;

        let mut keyboard =
            gpio::KeyEventStream::new().context("Failed to create gpio event stream")?;

        loop {
            select! {
                _ = keyboard_if.poll() => {}
                event = keyboard.next() => {
                    tx.send(event).ok();
                }
            }
        }
    }
}

mod gpio {
    use super::{Key, KeyEvent};
    use evdev::{Device, EventStream, InputEventKind};
    use std::{io, path::PathBuf};

    pub struct KeyEventStream {
        stream: EventStream,
    }

    const KB_PATH: &str = "gpio_keys";

    impl KeyEventStream {
        pub fn new() -> io::Result<Self> {
            let pred = |(_, dev): &(PathBuf, Device)| dev.name() == Some(KB_PATH);
            let find_err = io::Error::new(io::ErrorKind::NotFound, "Device not found");
            let dev = evdev::enumerate().find(pred).ok_or(find_err)?.1;
            let stream = dev.into_event_stream()?;
            Ok(Self { stream })
        }

        pub async fn next(&mut self) -> KeyEvent {
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
                            0 => break KeyEvent::Release(key),
                            1 => break KeyEvent::Press(key),
                            2 => break KeyEvent::Repeat(key),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
