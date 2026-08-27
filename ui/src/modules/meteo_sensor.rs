use crate::core::{interface::IfMngr, module::Module};
use crate::create_imc_interface;
use crate::modules::i2c::I2CIf;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Oversampling setting (0..=3). 0 = standard, 3 = ultra high resolution.
const OSS: u8 = 0;

create_imc_interface! {
    pub interface MeteoSensorIf {
        fn temperature() -> Result<f32>;
        fn pressure() -> Result<f32>;
    }
}

pub struct MeteoSensor {}

impl MeteoSensor {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for MeteoSensor {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let i2c: I2CIf = if_mngr.get("i2c").await.context("Failed to get i2c if")?;

        let bmp = bmp180::Bmp180::new(i2c)
            .await
            .context("Failed to init BMP180")?;
        let bmp = Arc::new(Mutex::new(bmp));

        let interface = MeteoSensorIfBackend::builder();

        let bmp_t = bmp.clone();
        let interface = interface.on_temperature(move || {
            let bmp = bmp_t.clone();
            async move { bmp.lock().await.temperature_c().await }
        });

        let bmp_p = bmp.clone();
        let interface = interface.on_pressure(move || {
            let bmp = bmp_p.clone();
            async move { bmp.lock().await.pressure_pa(OSS).await }
        });

        let mut interface = interface
            .build()
            .context("Failed to build meteo sensor interface")?;

        let interface_front = interface.get_if();
        if_mngr
            .reg("meteo_sensor", move || interface_front.clone())
            .await
            .context("Failed to reg meteo sensor interface")?;

        loop {
            interface.poll().await;
        }
    }
}

mod bmp180 {
    use crate::modules::i2c::I2CIf;
    use anyhow::Result;
    use embedded_hal_async::i2c::I2c;
    use std::time::Duration;

    const BMP180_ADDR: u8 = 0x77;

    const REG_CALIB_START: u8 = 0xAA;
    const REG_CTRL_MEAS: u8 = 0xF4;
    const REG_OUT_MSB: u8 = 0xF6;

    const CMD_READ_TEMP: u8 = 0x2E;
    const CMD_READ_PRESSURE_BASE: u8 = 0x34;

    #[derive(Debug, Clone, Copy, Default)]
    struct Calibration {
        ac1: i16,
        ac2: i16,
        ac3: i16,
        ac4: u16,
        ac5: u16,
        ac6: u16,
        b1: i16,
        b2: i16,
        mb: i16,
        mc: i16,
        md: i16,
    }

    pub struct Bmp180 {
        i2c: I2CIf,
        calib: Calibration,
    }

    impl Bmp180 {
        pub async fn new(mut i2c: I2CIf) -> Result<Self> {
            let calib = Self::read_calibration(&mut i2c).await?;
            Ok(Self { i2c, calib })
        }

        async fn read_calibration(i2c: &mut I2CIf) -> Result<Calibration> {
            let mut buf = [0u8; 22];
            i2c.write_read(BMP180_ADDR, &[REG_CALIB_START], &mut buf)
                .await
                .map_err(|e| anyhow::Error::msg(format!("{e:?}")))?;

            let rd_i16 = |hi: usize| i16::from_be_bytes([buf[hi], buf[hi + 1]]);
            let rd_u16 = |hi: usize| u16::from_be_bytes([buf[hi], buf[hi + 1]]);

            Ok(Calibration {
                ac1: rd_i16(0),
                ac2: rd_i16(2),
                ac3: rd_i16(4),
                ac4: rd_u16(6),
                ac5: rd_u16(8),
                ac6: rd_u16(10),
                b1: rd_i16(12),
                b2: rd_i16(14),
                mb: rd_i16(16),
                mc: rd_i16(18),
                md: rd_i16(20),
            })
        }

        async fn write_reg(&mut self, reg: u8, val: u8) -> Result<()> {
            self.i2c
                .write(BMP180_ADDR, &[reg, val])
                .await
                .map_err(|e| anyhow::Error::msg(format!("{e:?}")))
        }

        async fn read_regs(&mut self, reg: u8, buf: &mut [u8]) -> Result<()> {
            self.i2c
                .write_read(BMP180_ADDR, &[reg], buf)
                .await
                .map_err(|e| anyhow::Error::msg(format!("{e:?}")))
        }

        async fn read_raw_temperature(&mut self) -> Result<i32> {
            self.write_reg(REG_CTRL_MEAS, CMD_READ_TEMP).await?;
            tokio::time::sleep(Duration::from_millis(5)).await;

            let mut buf = [0u8; 2];
            self.read_regs(REG_OUT_MSB, &mut buf).await?;
            Ok(((buf[0] as i32) << 8) | (buf[1] as i32))
        }

        async fn read_raw_pressure(&mut self, oss: u8) -> Result<i32> {
            self.write_reg(REG_CTRL_MEAS, CMD_READ_PRESSURE_BASE + (oss << 6))
                .await?;

            let delay_ms = match oss {
                0 => 5,
                1 => 8,
                2 => 14,
                _ => 26,
            };
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            let mut buf = [0u8; 3];
            self.read_regs(REG_OUT_MSB, &mut buf).await?;
            let up =
                (((buf[0] as i32) << 16) | ((buf[1] as i32) << 8) | (buf[2] as i32)) >> (8 - oss);
            Ok(up)
        }

        fn compute_b5(&self, ut: i32) -> i32 {
            let c = &self.calib;
            let x1 = ((ut - c.ac6 as i32) * c.ac5 as i32) >> 15;
            let x2 = ((c.mc as i32) << 11) / (x1 + c.md as i32);
            x1 + x2
        }

        pub async fn temperature_c(&mut self) -> Result<f32> {
            let ut = self.read_raw_temperature().await?;
            let b5 = self.compute_b5(ut);
            let t = (b5 + 8) >> 4;
            Ok(t as f32 / 10.0)
        }

        pub async fn pressure_pa(&mut self, oss: u8) -> Result<f32> {
            let ut = self.read_raw_temperature().await?;
            let b5 = self.compute_b5(ut);

            let up = self.read_raw_pressure(oss).await?;
            let c = &self.calib;

            let b6 = b5 - 4000;
            let x1 = ((c.b2 as i32) * ((b6 * b6) >> 12)) >> 11;
            let x2 = ((c.ac2 as i32) * b6) >> 11;
            let x3 = x1 + x2;
            let b3 = ((((c.ac1 as i32) * 4 + x3) << oss) + 2) >> 2;

            let x1 = ((c.ac3 as i32) * b6) >> 13;
            let x2 = ((c.b1 as i32) * ((b6 * b6) >> 12)) >> 16;
            let x3 = (x1 + x2 + 2) >> 2;
            let b4 = ((c.ac4 as u32) * ((x3 + 32768) as u32)) >> 15;
            let b7 = ((up - b3) as u32) * (50000 >> oss);

            let p = if b7 < 0x8000_0000 {
                (b7 * 2) / b4
            } else {
                (b7 / b4) * 2
            } as i32;

            let x1 = (p >> 8) * (p >> 8);
            let x1 = (x1 * 3038) >> 16;
            let x2 = (-7357 * p) >> 16;
            let p = p + ((x1 + x2 + 3791) >> 4);

            Ok(p as f32)
        }
    }
}
