use crate::{
    core::{interface::IfMngr, module::Module},
    modules::{
        display::DisplayIf,
        keyboard::{KeyEvent, KeyboardIf},
    },
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

pub struct KeyboardTest {
    keys: Receiver<KeyEvent>,
    display: DisplayIf,
}

impl KeyboardTest {
    pub async fn build(if_mngr: IfMngr) -> Result<Self> {
        let keys = if_mngr
            .get::<&str, KeyboardIf>("keyboard")
            .await
            .context("Failed to get keyboard interface")?
            .subscribe()
            .await
            .context("Failed to subscribe to keyboard events")?;
        let display = if_mngr
            .get::<&str, DisplayIf>("display")
            .await
            .context("Failed to get display interface")?;
        Ok(Self { keys, display })
    }
}

#[async_trait]
impl Module for KeyboardTest {
    async fn run(&mut self, _if_mngr: IfMngr) -> Result<()> {
        self.display
            .flush()
            .await
            .context("Failed to flush display")?;
        loop {
            let key = self
                .keys
                .recv()
                .await
                .context("Failed to receive key event")?;
            self.display
                .clear()
                .await
                .context("Failed to clear display")?;
            self.display
                .draw_line(format!("{:?}", key), 0)
                .await
                .context("Failed to display text")?;
            self.display
                .flush()
                .await
                .context("Failed to flush display")?;
        }
    }
}
