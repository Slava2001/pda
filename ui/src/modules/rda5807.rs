use crate::core::{interface::IfMngr, module::Module};
use crate::create_imc_interface;
use crate::modules::i2c::I2CIf;
use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use embedded_hal_async::i2c::I2c;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

create_imc_interface! {
    pub interface Rda5807If {
        fn enable() -> Result<()>;
        fn disable() -> Result<()>;
        fn set_freq(freq: f32) -> Result<()>;
        fn set_volume(volume: u8) -> Result<()>;
        fn get_freq() -> Result<f32>;
        fn get_rssi() -> Result<f32>;
    }
}

pub struct Rda5807 {}

impl Rda5807 {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for Rda5807 {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let i2c: I2CIf = if_mngr.get("i2c").await.context("Failed to get i2c if")?;
        let mut chip = RDA5807Driver::new(i2c);
        chip.init().await?;
        let chip = Arc::new(Mutex::new(chip));

        let interface = Rda5807IfBackend::builder();
        let chip_c = chip.clone();
        let interface = interface.on_enable(move || {
            let chip = chip_c.clone();
            async move { chip.lock().await.enable().await }
        });
        let chip_c = chip.clone();
        let interface = interface.on_disable(move || {
            let chip = chip_c.clone();
            async move { chip.lock().await.disable().await }
        });
        let chip_c = chip.clone();
        let interface = interface.on_set_freq(move |freq| {
            let chip = chip_c.clone();
            async move { chip.lock().await.set_freq(freq).await }
        });
        let chip_c = chip.clone();
        let interface = interface.on_set_volume(move |volume| {
            let chip = chip_c.clone();
            async move { chip.lock().await.set_volume(volume).await }
        });
        let chip_c = chip.clone();
        let interface = interface.on_get_freq(move || {
            let chip = chip_c.clone();
            async move { chip.lock().await.get_freq().await }
        });
        let interface = interface.on_get_rssi(move || {
            let chip = chip.clone();
            async move { chip.lock().await.get_rssi().await }
        });
        let mut interface = interface
            .build()
            .context("Failed to build rda5807 controller interface")?;

        let interface_front = interface.get_if();
        if_mngr
            .reg("rda5807", move || interface_front.clone())
            .await
            .context("Failed to reg rda5807 controller interface")?;

        loop {
            let event = interface.poll_event().await;
            interface.handle_event(event).await;
        }
    }
}

const RDA5807_I2C_ADDR_RAND_ACCESS: u8 = 0x11;
const MAX_VOLUME: u8 = 15;

// Registers
// REG 0x02 — Basic Control
//
// [15]    DHIZ
//         0 = Audio output high-Z
//         1 = Normal operation
//
// [14]    DMUTE
//         0 = Mute
//         1 = Normal audio
//
// [13]    MONO
//         0 = Stereo
//         1 = Force mono
//
// [12]    BASS
//         0 = Bass boost disabled
//         1 = Bass boost enabled
//
// [11]    NON_CALIBRATE
//         0 = RCLK calibration enabled
//         1 = RCLK non-calibrate
//
// [10]    RCLK_DIRECT_IN
//         0 = RCLK from oscillator
//         1 = RCLK direct input
//
// [9]     SEEKUP
//         0 = Seek down
//         1 = Seek up
//
// [8]     SEEK
//         0 = Seek disabled
//         1 = Start seek
//
// [7]     SKMODE
//         0 = Seek wraps around at band limit
//         1 = Seek stops at band limit
//
// [6:4]   CLK_MODE
//         000 = 32.768 kHz
//         001 = 12 MHz
//         010 = 13 MHz
//         011 = 19.2 MHz
//         100 = Reserved
//         101 = 24 MHz
//         110 = 26 MHz
//         111 = 38.4 MHz
//
// [3]     RDS_EN
//         0 = RDS disabled
//         1 = RDS enabled
//
// [2]     NEW_METHOD
//         0 = Old demodulation method
//         1 = New demodulation method
//
// [1]     SOFT_RESET
//         0 = Normal
//         1 = Software reset
//
// [0]     ENABLE
//         0 = Power down
//         1 = Power up
//
const REG_BASIC_CTRL: u8 = 0x02;

// REG 0x03 — Channel / Tuning
//
// [15:6]  CHAN
//         Channel number, 0..1023
//
//         Frequency:
//           BAND=00:
//             F = 87.0 MHz + CHAN * SPACE
//
//           BAND=01/10:
//             F = 76.0 MHz + CHAN * SPACE
//
//           BAND=11:
//             F = 65.0 MHz + CHAN * SPACE
//
// [5]     DIRECT_MODE
//         0 = Normal
//         1 = Direct/test mode
//
// [4]     TUNE
//         0 = No tuning
//         1 = Start tuning
//
//         Automatically cleared after tuning.
//
// [3:2]   BAND
//         00 = 87..108 MHz
//         01 = 76..91 MHz
//         10 = 76..108 MHz
//         11 = 65..76 MHz / 50..76 MHz
//
//         For BAND=11 the exact range depends on REG07[9].
//
// [1:0]   SPACE
//         00 = 100 kHz
//         01 = 200 kHz
//         10 = 50 kHz
//         11 = 25 kHz
//
const REG_CHANNEL: u8 = 0x03;

// REG 0x04 — Receiver / GPIO / RDS / I2S
//
// [15]    Reserved
//
// [14]    STCIEN
//         0 = STC interrupt disabled
//         1 = STC interrupt enabled
//
// [13]    RBDS
//         0 = RDS mode
//         1 = RBDS mode
//
// [12]    RDS_FIFO_EN
//         0 = RDS FIFO disabled
//         1 = RDS FIFO enabled
//
// [11]    DE
//         0 = De-emphasis 75 us
//         1 = De-emphasis 50 us
//
// [10]    RDS_FIFO_CLR
//         0 = Normal
//         1 = Clear RDS FIFO
//
// [9]     SOFTMUTE_EN
//         0 = Soft mute disabled
//         1 = Soft mute enabled
//
// [8]     AFCD
//         0 = AFC enabled
//         1 = AFC disabled
//
// [7]     Reserved
//
// [6]     I2S_ENABLE
//         0 = I2S disabled
//         1 = I2S enabled
//
// [5:4]   GPIO3
//         00 = High-Z
//         01 = Stereo indicator
//         10 = Low
//         11 = High
//
// [3:2]   GPIO2
//         00 = High-Z
//         01 = Reserved
//         10 = Low
//         11 = High
//
// [1:0]   GPIO1
//         00 = High-Z
//         01 = Reserved
//         10 = Low
//         11 = High
//
const REG_CONFIG: u8 = 0x04;

// REG 0x05 — Seek / LNA / Volume
//
// [15]    INT_MODE
//         0 = Generate 5 ms interrupt
//         1 = Generate interrupt according to STCIEN
//
// [14:13] SEEK_MODE
//         00 = Normal seek mode
//         01 = Reserved / legacy mode
//         10 = RSSI based seek mode
//         11 = Reserved
//
// [12]    Reserved
//
// [11:8]  SEEKTH
//         Seek RSSI threshold
//         0x0 = minimum threshold
//         0xF = maximum threshold
//
// [7:6]   LNA_PORT_SEL
//         00 = No input
//         01 = LNAN
//         10 = LNAP
//         11 = Dual port
//
// [5:4]   LNA_ICSEL_BIT
//         00 = 1.8 mA
//         01 = 2.1 mA
//         10 = 2.5 mA
//         11 = 3.0 mA
//
// [3:0]   VOLUME
//         0x0 = Minimum
//         ...
//         0xF = Maximum
//
const REG_VOLUME: u8 = 0x05;

// REG 0x06 — I2S Configuration
//
// [15]    Reserved
//
// [14:13] OPEN_MODE
//         00 = Open behind-register read
//         01 = Open behind-register read
//         10 = Open behind-register read
//         11 = Open behind-register write
//
// [12]    SLAVE_MASTER
//         0 = I2S master
//         1 = I2S slave
//
// [11]    WS_LR
//         0 = Left/right channel configuration 0
//         1 = Left/right channel configuration 1
//
// [10]    SCLK_I_EDGE
//         0 = Sample on falling edge
//         1 = Sample on rising edge
//
// [9]     DATA_SIGNED
//         0 = Unsigned data
//         1 = Signed data
//
// [8]     WS_I_EDGE
//         0 = WS normal
//         1 = WS inverted
//
// [7:4]   I2S_SW_CNT
//         0000 = 8 kHz
//         0001 = 11.025 kHz
//         0010 = 12 kHz
//         0011 = 16 kHz
//         0100 = 22.05 kHz
//         0101 = 24 kHz
//         0110 = 32 kHz
//         0111 = 44.1 kHz
//         1000 = 48 kHz
//
// [3]     SW_O_EDGE
//         0 = Normal
//         1 = Inverted
//
// [2]     SCLK_O_EDGE
//         0 = Normal
//         1 = Inverted
//
// [1]     L_DELY
//         0 = No delay
//         1 = Delay
//
// [0]     R_DELY
//         0 = No delay
//         1 = Delay
//
const REG_I2S: u8 = 0x06;

// REG 0x07 — System Configuration
//
// [15]    Reserved
//
// [14:10] TH_SOFRBLEND
//         Soft-blend threshold
//         Unit = 2 dB
//
// [9]     MODE_50_60
//         0 = BAND=11 → 50..76 MHz
//         1 = BAND=11 → 65..76 MHz
//
// [8]     Reserved
//
// [7:2]   SEEK_TH_OLD
//         Old seek RSSI threshold
//         Used when SEEK_MODE = 001
//
// [1]     SOFTBLEND_EN
//         0 = Soft blend disabled
//         1 = Soft blend enabled
//
// [0]     FREQ_MODE
//         0 = Normal CHAN frequency mode
//         1 = Direct frequency mode
//
//         When FREQ_MODE=1, frequency is set using REG08.
//
const REG_SYSTEM: u8 = 0x07;

// REG 0x0A — Status 1 (read-only)
//
// [15]    RDSR
//         0 = No new RDS group
//         1 = New RDS/RBDS group ready
//
// [14]    STC
//         0 = Tune/seek not complete
//         1 = Tune/seek complete
//
// [13]    SF
//         0 = Seek successful
//         1 = Seek failed
//
// [12]    RDSS
//         0 = RDS not synchronized
//         1 = RDS synchronized
//
// [11]    BLK_E
//         RDS block E found
//
// [10]    ST
//         0 = Mono
//         1 = Stereo
//
// [9:0]   READCHAN
//         Current channel number
//
//         Frequency:
//           BAND=00:
//             F = 87.0 MHz + READCHAN * SPACE
//
//           BAND=01/10:
//             F = 76.0 MHz + READCHAN * SPACE
//
//           BAND=11:
//             F = 65.0 MHz + READCHAN * SPACE
//
const REG_STATUS1: u8 = 0x0A;

// REG 0x0B — Status 2 (read-only)
//
// [15:9]  RSSI
//         Received Signal Strength Indicator
//         0 = minimum
//         127 = maximum
//         Logarithmic scale
//
// [8]     FM_TRUE
//         0 = Current channel is not a station
//         1 = Current channel is a valid station
//
// [7]     FM_READY
//         0 = FM not ready
//         1 = FM ready
//
// [6:5]   Reserved
//
// [4]     ABCD_E
//         0 = RDS blocks are A/B/C/D
//         1 = RDS block is E
//
// [3:2]   BLERA
//         00 = 0 errors
//         01 = 1..2 errors
//         10 = 3..5 errors
//         11 = 6+ errors / uncorrectable
//
// [1:0]   BLERB
//         00 = 0 errors
//         01 = 1..2 errors
//         10 = 3..5 errors
//         11 = 6+ errors / uncorrectable
//
const REG_STATUS2: u8 = 0x0B;

const REG_RDS_BLOCK_A: u8 = 0x0C;
const REG_RDS_BLOCK_B: u8 = 0x0D;
const REG_RDS_BLOCK_C: u8 = 0x0E;
const REG_RDS_BLOCK_D: u8 = 0x0F;

pub struct RDA5807Driver<I2C> {
    i2c: I2C,
    band_start_mhz: f32,
    band_end_mhz: f32,
    band_step_mhz: f32,
}

impl<I2C> RDA5807Driver<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            band_start_mhz: 0.0,
            band_end_mhz: 0.0,
            band_step_mhz: 0.0,
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        self.write_reg(REG_BASIC_CTRL, 0xC001).await?;
        self.write_reg(REG_CHANNEL, 0x0000).await?;
        self.write_reg(REG_CONFIG, 0x0000).await?;
        self.write_reg(REG_VOLUME, 0x8800).await?;
        self.band_start_mhz = 87.0;
        self.band_end_mhz = 108.0;
        self.band_step_mhz = 0.1;
        Ok(())
    }

    pub async fn enable(&mut self) -> Result<()> {
        self.modify_reg(REG_BASIC_CTRL, |mut reg| {
            reg |= 0x0001;
            reg
        }).await?;
        Ok(())
    }

    pub async fn disable(&mut self) -> Result<()> {
        self.modify_reg(REG_BASIC_CTRL, |mut reg| {
            reg &= !0x0001;
            reg
        }).await?;
        Ok(())
    }

    pub async fn set_volume(&mut self, volume: u8) -> Result<()> {
        let volume = volume.min(MAX_VOLUME);
        self.modify_reg(REG_VOLUME, |mut reg| {
            reg &= !0x000F;
            reg |= volume as u16;
            reg
        }).await?;
        Ok(())
    }

    pub async fn set_freq(&mut self, mut freq: f32) -> Result<()> {
        freq = freq.clamp(self.band_start_mhz, self.band_end_mhz);
        let channel = ((freq - self.band_start_mhz) / self.band_step_mhz).round() as u16;
        self.modify_reg(REG_CHANNEL, |mut reg| {
            reg &= !(0x03FF << 6);
            reg |= channel << 6;
            reg |= 1 << 4;
            reg
        })
        .await?;
        self.wait_stc().await?;
        Ok(())
    }

    pub async fn get_freq(&mut self) -> Result<f32> {
        let reg = self.read_reg(REG_STATUS1).await?;
        let channel = reg & 0x03FF;
        Ok(self.band_start_mhz + channel as f32 * self.band_step_mhz)
    }

    pub async fn get_rssi(&mut self) -> Result<f32> {
        let reg = self.read_reg(REG_STATUS2).await?;
        let rssi = (reg >> 9) & 0x7F;
        Ok(rssi as f32)
    }

    async fn wait_stc(&mut self) -> Result<()> {
        loop {
            let status = self.read_reg(REG_STATUS1).await?;
            if status & (1 << 14) != 0 {
                return Ok(());
            }
            sleep(Duration::from_millis(5)).await;
        }
    }

    async fn modify_reg<F>(&mut self, addr: u8, f: F) -> Result<()>
    where
        F: FnOnce(u16) -> u16,
    {
        let reg = self.read_reg(addr).await?;
        self.write_reg(addr, f(reg)).await
    }

    async fn write_reg(&mut self, addr: u8, val: u16) -> Result<()> {
        let data = [addr, (val >> 8) as u8, val as u8];

        self.i2c
            .write(RDA5807_I2C_ADDR_RAND_ACCESS, &data)
            .await
            .map_err(|e| Error::msg(format!("I2c error: {e:?}")))?;

        Ok(())
    }

    async fn read_reg(&mut self, addr: u8) -> Result<u16> {
        let mut data = [0u8; 2];

        self.i2c
            .write_read(RDA5807_I2C_ADDR_RAND_ACCESS, &[addr], &mut data)
            .await
            .map_err(|e| Error::msg(format!("I2c error: {e:?}")))?;

        Ok(u16::from_be_bytes(data))
    }
}
