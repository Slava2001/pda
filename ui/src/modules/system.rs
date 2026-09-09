use crate::core::{interface::IfMngr, module::Module};
use crate::create_imc_interface;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::interval;

create_imc_interface! {
    pub interface SystemIf {
        fn cpu_temp() -> f32;
    }
}

pub struct System {}

impl System {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for System {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let temperature = Arc::new(Mutex::new(0.0 as f32));

        let interface = SystemIfBackend::builder();
        let temp_c = temperature.clone();
        let interface = interface.on_cpu_temp(move || {
            let temp = temp_c.clone();
            async move { *temp.lock().await }
        });

        let mut interface = interface
            .build()
            .context("Failed to build system interface")?;

        let interface_front = interface.get_if();
        if_mngr
            .reg("system", move || interface_front.clone())
            .await
            .context("Failed to reg system interface")?;

        let mut timer = interval(Duration::from_secs_f32(0.5));
        loop {
            select! {
                event = interface.poll_event() => {
                    interface.handle_event(event).await;
                }
                _ = timer.tick() => {
                    const THERMAL_ZONE_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
                    if let Ok(raw) = tokio::fs::read_to_string(THERMAL_ZONE_PATH).await {
                        let millideg: i32 = raw.trim().parse()
                                                .context("Failed to parse thermal zone value")?;
                        *temperature.lock().await = millideg as f32 / 1000.0;
                    }
                }
            }
        }
    }
}
