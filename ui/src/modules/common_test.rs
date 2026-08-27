use std::time::Duration;

use crate::{
    core::{interface::IfMngr, module::Module},
    modules::{
        display::DisplayIf,
        keyboard::{KeyEvent, KeyboardIf},
        meteo_sensor::MeteoSensorIf,
        net_ctrl::NetCtrlIf,
        power_ctrl::PowerCtrlIf,
        system::SystemIf
    },
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::{select, sync::broadcast::{Receiver, error::RecvError}, time::interval};

pub struct CommonTest {
    keys: Receiver<KeyEvent>,
    display: DisplayIf,
    power_ctrl: PowerCtrlIf,
    net_ctrl: NetCtrlIf,
    meteo_sensor: MeteoSensorIf,
    system: SystemIf
}

impl CommonTest {
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
        let power_ctrl = if_mngr
            .get("power_ctrl")
            .await
            .context("Failed to get power controller interface")?;
        let net_ctrl = if_mngr
            .get("net_ctrl")
            .await
            .context("Failed to get net controller interface")?;
        let meteo_sensor = if_mngr
            .get("meteo_sensor")
            .await
            .context("Failed to get net controller interface")?;
        let system = if_mngr
            .get("system")
            .await
            .context("Failed to get system interface")?;
        Ok(Self {
            keys,
            display,
            power_ctrl,
            net_ctrl,
            meteo_sensor,
            system
        })
    }
}

#[async_trait]
impl Module for CommonTest {
    async fn run(&mut self, _if_mngr: IfMngr) -> Result<()> {
        self.display.draw_line(format!("IP: -"), 0).await?;
        self.display
            .draw_line(format!("Power controller:"), 1)
            .await?;
        self.display.draw_line(format!("Keyboard Test:"), 4).await?;
        self.display
            .draw_line(format!("Press any button..."), 5)
            .await?;
        self.display
            .draw_line(format!("Meteo sensor:"), 6)
            .await?;
        self.display
            .draw_line(format!("System:"), 10)
            .await?;
        self.display
            .flush()
            .await
            .context("Failed to flush display")?;
        let mut timer = interval(Duration::from_micros(1000 / 60));
        loop {
            select! {
                res = self.keys.recv() => {
                    match res {
                        Ok(key) => {
                            self.display.draw_line(format!("{:?}", key), 5)
                                .await?;
                        }
                        Err(RecvError::Lagged(_)) => {}
                        err => {
                            err.context("Failed to receive key event")?;
                        }
                    }
                }
                _ = timer.tick() => {
                    let ip = self.net_ctrl.get_ip().await?
                                 .map(|ip| format!("{ip}")).unwrap_or("-".into());
                    self.display.draw_line(format!("IP: {}", ip), 0)
                        .await?;

                    let voltage = self.power_ctrl.voltage().await?.context("Failed to get voltage")?;
                    self.display.draw_line(format!("Voltage: {:.4} V", voltage), 2)
                        .await?;

                    let current = self.power_ctrl.current().await?.context("Failed to get current")?;
                    self.display.draw_line(format!("Current: {:.4} A", current), 3)
                        .await?;

                    let temperature = self.meteo_sensor.temperature().await?;
                    self.display.draw_line(format!("Temperature: {:.1} *C", temperature), 7)
                        .await?;

                    let pressure_raw = self.meteo_sensor.pressure_raw().await?;
                    self.display.draw_line(format!("Pressure_raw: {} Pa", pressure_raw), 8)
                        .await?;

                    let pressure = self.meteo_sensor.pressure().await?;
                    self.display.draw_line(format!("Pressure: {:.0} Pa", pressure), 9)
                        .await?;

                    let cpu_temp = self.system.cpu_temp().await?;
                    self.display.draw_line(format!("CPU temp: {:.1} *C", cpu_temp), 11)
                        .await?;
                }
            }

            self.display
                .flush()
                .await
                .context("Failed to flush display")?;
        }
    }
}
