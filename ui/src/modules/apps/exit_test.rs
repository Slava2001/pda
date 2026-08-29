use crate::{
    core::{interface::IfMngr, module::Module},
    modules::{
        display::DisplayIf,
        keyboard::{KeyEvent, KeyboardIf},
    },
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::{
    select,
    sync::broadcast::{Receiver, error::RecvError},
};

pub struct ExitTest {
    keys: Receiver<KeyEvent>,
    display: DisplayIf,
}

impl ExitTest {
    pub async fn build(if_mngr: IfMngr) -> Result<Self> {
        let keys = if_mngr
            .get::<&str, KeyboardIf>("keyboard")
            .await
            .context("Failed to get keyboard interface")?
            .subscribe()
            .await
            .context("Failed to subscribe to keyboard events")?;
        let display = if_mngr
            .get("display")
            .await
            .context("Failed to get display interface")?;
        Ok(Self { keys, display })
    }
}

#[async_trait]
impl Module for ExitTest {
    async fn run(&mut self, _if_mngr: IfMngr) -> Result<()> {
        let txt = "Press any button to exit".to_string();
        self.display.draw_line(txt, 0).await?;
        self.display.flush().await?;
        loop {
            select! {
                res = self.keys.recv() => {
                    match res {
                        Ok(KeyEvent::Press(_key)) => {
                            return Ok(())
                        }
                        Ok(_) |
                        Err(RecvError::Lagged(_)) => {}
                        err => {
                            err.context("Failed to receive key event")?;
                        }
                    }
                }
            }
        }
    }
}
