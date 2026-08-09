pub mod cmd;
pub mod error;

use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;
use linux_embedded_hal::gpio_cdev::LineHandle;
use std::println;

use crate::si4732::{
    cmd::{Cmd, Func, LibraryID, OpMode, Property, Response, Status, TuneStatus}, error::{Context, Error},
};

pub struct Si4732<I2C, D> {
    i2c: I2C,
    reset: LineHandle,
    address: u8,
    delay: D,
    library_id: Option<LibraryID>,
}

impl<I2C, D> Si4732<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    pub fn new(i2c: I2C, delay: D, reset: LineHandle, address: u8) -> Self {
        Self {
            i2c,
            reset,
            address,
            delay,
            library_id: None,
        }
    }

    pub fn get_part_number(&mut self) -> Result<u8, Error> {
        Ok(self.read_library_id()?.part_number)
    }

    pub fn get_library_id(&mut self) -> Result<u8, Error> {
        Ok(self.read_library_id()?.library_id)
    }

    pub fn get_firmware_ver(&mut self) -> Result<String, Error> {
        Ok(self.read_library_id()?.firmware.iter().collect())
    }

    pub fn get_chip_rev(&mut self) -> Result<String, Error> {
        Ok(self.read_library_id()?.chip_revision.to_string())
    }

    pub fn set_volume(&mut self, val: u8) -> Result<(), Error> {
        if val > 63 {
            return Err(Error::from("volume out of range"));
        }
        self.set_property(Property::RxVolume, val.into())?;
        self.set_property(Property::RxHardMute, 0)
    }

    pub fn set_frequency(&mut self, freq_khz: u32) -> Result<(), Error> {
        self.send_cmd(Cmd::FmTuneFreq {
            freeze: true,
            fast: true,
            freq: (freq_khz / 10) as u16,
            antcap: 0,
        })?;
        self.wait_cts()
    }

    pub fn get_rssi_and_snr(&mut self) -> Result<(u8, u8), Error> {
        self.send_cmd(Cmd::FmTuneStatus { cancel: false, int_ack: false })?;
        self.wait_cts()?;
        let status: TuneStatus = self.read_resp()?;
        println!("{status:#?}");
        Ok((status.rssi, status.snr))
    }

    pub fn init(&mut self, func: Func) -> Result<(), Error> {
        self.reset()?;
        self.send_cmd(Cmd::PowerUp {
            cts_interrupt_enable: false,
            gpo2_output_enable: true,
            patch_enable: false,
            crystal_oscillator_enable: true,
            func,
            opmode: OpMode::AnalogAudioOutputs,
        })?;
        self.wait_cts()?;
        Ok(())
    }

    fn set_property(&mut self, prop: Property, val: u16) -> Result<(), Error> {
        self.send_cmd(Cmd::SetProperty { prop, val })?;
        self.wait_cts()
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.reset.set_value(0).cc("set reset pin to 0")?;
        self.delay.delay_ms(10);
        self.reset.set_value(1).cc("set reset pin to 1")?;
        self.delay.delay_ms(10);
        Ok(())
    }

    fn send_cmd(&mut self, cmd: Cmd) -> Result<(), Error> {
        let mut buf = [0u8; 8];
        let len = cmd.encode(&mut buf);
        println!("{}: {:?}", len, buf);
        self.i2c
            .write(self.address, &buf[..len])
            .cc("write bytes")?;
        Ok(())
    }

    fn read_resp<const L: usize, T: Response<L>>(&mut self) -> Result<T, Error> {
        let mut buf = [0u8; L];
        self.i2c
            .read(self.address, &mut buf)
            .cc("read response bytes")?;
        Ok(T::form_bytes(&buf))
    }

    fn wait_cts(&mut self) -> Result<(), Error> {
        self.delay.delay_ms(100);
        for _ in 0..1000 {
            // self.send_cmd(Cmd::GetIntStatus)?;
            let status: Status = self.read_resp()?;
            println!("clear_to_send: {}", status.clear_to_send);
            if status.clear_to_send {
                return Ok(());
            }
            self.delay.delay_ms(10);
        }
        Err(()).cc("wait rc: too long delay")
    }

    fn read_library_id(&mut self) -> Result<&LibraryID, Error> {
        if self.library_id.is_none() {
            self.reset()?;
            self.send_cmd(Cmd::PowerUp {
                cts_interrupt_enable: false,
                gpo2_output_enable: true,
                patch_enable: false,
                crystal_oscillator_enable: true,
                func: Func::QueryLibraryID,
                opmode: OpMode::AnalogAudioOutputs,
            })?;
            self.wait_cts()?;
            self.library_id = Some(self.read_resp()?);
        }
        self.library_id
            .as_ref()
            .ok_or("get lib id: unexpected".into())
    }
}
