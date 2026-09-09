use std::time::Duration;

use crate::{
    core::{interface::IfMngr, module::Module},
    modules::{
        display::{DisplayIf, Rect},
        keyboard::{Key, KeyEvent, KeyboardIf},
        rda5807::Rda5807If,
    },
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::{
    select, spawn, sync::broadcast::{Receiver, error::RecvError}, time::interval,
};

pub struct Radio {
    keys: Receiver<KeyEvent>,
    rda5807: Rda5807If,
    display: DisplayIf,
}

impl Radio {
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
        let rda5807 = if_mngr
            .get("rda5807")
            .await
            .context("Failed to get Rda5807 interface")?;
        Ok(Self {
            keys,
            display,
            rda5807,
        })
    }
}

#[async_trait]
impl Module for Radio {
    async fn run(&mut self, _if_mngr: IfMngr) -> Result<()> {
        let mut volume = 0;
        let mut frequency = 89.8;

        self.rda5807.enable().await??;
        self.rda5807.set_volume(volume).await??;
        self.rda5807.set_freq(frequency).await??;

        let mut data = vec![0.0; 100];
        loop {
            frequency = self.rda5807.get_freq().await??;
            self.display
                .draw_line(format!("     ___RDA5807___"), 0)
                .await?;
            self.display
                .draw_line(format!("Volume: {volume}"), 1)
                .await?;
            self.display
                .draw_line(format!("Freq: {frequency}"), 2)
                .await?;
            self.display
                .draw_graph(
                    Rect {
                        x: 10,
                        y: 90,
                        width: 220,
                        height: 220,
                    },
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: data.len() as f32,
                        height: 128.0,
                    },
                    data.clone(),
                )
                .await?;
            self.display
                .flush()
                .await
                .context("Failed to flush display")?;

            let mut timer = interval(Duration::from_secs_f32(1.0/20.0));
            select! {
                res = self.keys.recv() => {
                    match res {
                        Ok(KeyEvent::Press(key)) |
                        Ok(KeyEvent::Repeat(key)) => {
                            match key {
                                Key::Up if volume < 15 => volume += 1,
                                Key::Down if volume > 0 => volume -= 1,
                                Key::Left => frequency -= 0.1,
                                Key::Right => frequency += 0.1,
                                _ => {}
                            }
                            self.rda5807.set_freq(frequency).await??;
                            self.rda5807.set_volume(volume).await??;
                        }
                        Ok(_) |
                        Err(RecvError::Lagged(_)) => {}
                        err => {
                            err.context("Failed to receive key event")?;
                        }
                    }
                }
                _ = timer.tick() => {
                    data.push(self.rda5807.get_rssi().await??);
                    if data.len() > 100 {
                        data.remove(0);
                    }
                }
            }
        }
    }
}

impl Drop for Radio {
    fn drop(&mut self) {
        let mut rda = self.rda5807.clone();
        spawn(async move { rda.disable().await.ok(); });
    }
}
