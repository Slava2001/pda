use std::time::Duration;

use crate::{
    core::{interface::IfMngr, module::Module},
    modules::{
        display::DisplayIf,
        keyboard::{KeyEvent, KeyboardIf},
        meteo_sensor::MeteoSensorIf,
    },
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::{
    select,
    sync::broadcast::{Receiver, error::RecvError},
    time::interval,
};

pub struct Barometer {
    keys: Receiver<KeyEvent>,
    meteo_sensor: MeteoSensorIf,
    display: DisplayIf,
}

impl Barometer {
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
        let meteo_sensor = if_mngr
            .get("meteo_sensor")
            .await
            .context("Failed to get net controller interface")?;
        Ok(Self {
            keys,
            display,
            meteo_sensor,
        })
    }
}

#[async_trait]
impl Module for Barometer {
    async fn run(&mut self, _if_mngr: IfMngr) -> Result<()> {
        let txt = "Press any button to exit".to_string();
        self.display.draw_line(txt, 0).await?;
        self.display.flush().await?;
        let mut timer = interval(Duration::from_micros(1000 / 60));

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
              _ = timer.tick() => {
                    let temperature = self.meteo_sensor.temperature().await?;
                    self.display.draw_line(format!("Temperature: {:.1} *C", temperature), 0)
                        .await?;
                    let pressure_raw = self.meteo_sensor.pressure_raw().await?;
                    self.display.draw_line(format!("Pressure_raw: {} Pa", pressure_raw), 1)
                        .await?;
                    let pressure = self.meteo_sensor.pressure().await?;
                    self.display.draw_line(format!("Pressure: {:.0} Pa", pressure), 2)
                        .await?;

                }
            }
        }
    }
}
