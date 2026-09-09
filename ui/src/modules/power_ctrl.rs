use crate::core::{interface::IfMngr, module::Module};
use crate::create_imc_interface;
use crate::modules::i2c::I2CIf;
use crate::utils::filter::LowPass;
use anyhow::{Context, Result};
use async_trait::async_trait;
use ina219::AsyncIna219;
use ina219::address::Address;
use ina219::configuration::{
    BusVoltageRange, Configuration, MeasuredSignals, OperatingMode, Reset, Resolution,
    ShuntVoltageRange,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::interval;

create_imc_interface! {
    pub interface PowerCtrlIf {
        fn get_charge() -> Result<f32>;
        fn get_voltage() -> Result<f32>;
        fn get_current() -> Result<f32>;
    }
}

pub struct PowerCtrl {}

impl PowerCtrl {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for PowerCtrl {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let i2c: I2CIf = if_mngr.get("i2c").await.context("Failed to get i2c if")?;
        let addr = Address::from_byte(0x40).map_err(|e| anyhow::Error::msg(format!("{e}")))?;
        let mut ina = AsyncIna219::new(i2c, addr)
            .await
            .map_err(|e| anyhow::Error::msg(format!("{e}")))?;
        ina.set_configuration(Configuration {
            bus_resolution: Resolution::Avg128,
            shunt_resolution: Resolution::Avg128,
            bus_voltage_range: BusVoltageRange::Fsr16v,
            shunt_voltage_range: ShuntVoltageRange::Fsr160mv,
            operating_mode: OperatingMode::Continous(MeasuredSignals::ShutAndBusVoltage),
            reset: Reset::Run,
        })
        .await
        .map_err(|e| anyhow::Error::msg(format!("{e:?}")))?;
        tokio::time::sleep(Duration::from_micros(
            ina.configuration()
                .await
                .map_err(|e| anyhow::Error::msg(format!("{e}")))?
                .conversion_time_us()
                .ok_or(anyhow::Error::msg(
                    "Failed to convert conversion time to us",
                ))? as u64,
        ))
        .await;

        let ina = Arc::new(Mutex::new(ina));
        let interface = PowerCtrlIfBackend::builder();
        let get_charge = |voltage: f32| -> f32 {
            const MAX_VOLTAGE: f32 = 4.05;
            const MIN_VOLTAGE: f32 = 3.20;
            let charge =
                0.0 + (100.0 - 0.0) * (voltage - MIN_VOLTAGE) / (MAX_VOLTAGE - MIN_VOLTAGE);
            charge.clamp(1.0, 100.0)
        };
        let voltage = ina
            .lock()
            .await
            .bus_voltage()
            .await
            .map_err(|e| anyhow::Error::msg(format!("{e}")))?
            .voltage_mv() as f32
            / 1_000.0;
        let charge = Arc::new(Mutex::new(LowPass::new(0.05, get_charge(voltage))));
        let charge_c = charge.clone();
        let interface = interface.on_get_charge(move || {
            let charge = charge_c.clone();
            async move { Ok(charge.lock().await.value()) }
        });
        let ina_c = ina.clone();
        let interface = interface.on_get_voltage(move || {
            let ina = ina_c.clone();
            async move {
                Ok(ina
                    .lock()
                    .await
                    .bus_voltage()
                    .await
                    .map_err(|e| anyhow::Error::msg(format!("{e}")))?
                    .voltage_mv() as f32
                    / 1_000.0)
            }
        });
        let ina_c = ina.clone();
        let interface = interface.on_get_current(move || {
            let ina = ina_c.clone();
            async move {
                Ok(ina
                    .lock()
                    .await
                    .shunt_voltage()
                    .await
                    .map_err(|e| anyhow::Error::msg(format!("{e}")))?
                    .shunt_voltage_uv() as f32
                    / 100_000.0)
            }
        });
        let mut interface = interface
            .build()
            .context("Failed to build power controller interface")?;

        let interface_front = interface.get_if();
        if_mngr
            .reg("power_ctrl", move || interface_front.clone())
            .await
            .context("Failed to reg power controller interface")?;
        let mut timer = interval(Duration::from_secs(1));
        loop {
            select! {
                event = interface.poll_event() => {
                    interface.handle_event(event).await;
                }
                _ = timer.tick() => {
                    let voltage = ina
                        .lock()
                        .await
                        .bus_voltage()
                        .await
                        .map_err(|e| anyhow::Error::msg(format!("{e}")))?
                        .voltage_mv() as f32
                        / 1_000.0;
                    charge.lock().await.update(get_charge(voltage));
                }
            }
        }
    }
}
