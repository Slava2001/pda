use crate::{
    core::{interface::IfMngr, module::Module},
    create_imc_interface,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use embedded_hal::i2c::I2c as _;
use embedded_hal_async::i2c::{ErrorType, I2c};
use linux_embedded_hal::I2cdev;
use std::{sync::Arc, vec};
use tokio::sync::Mutex;

pub enum I2CIfOps {
    Write(Vec<u8>),
    Read(usize),
}

#[derive(Debug)]
pub struct I2CIfError;

create_imc_interface! {
    pub interface I2CIf {
        fn rw_data(addr: u8, ops: Vec<I2CIfOps>) -> Result<Vec<Vec<u8>>>;
    }
}

impl embedded_hal_async::i2c::Error for I2CIfError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

impl ErrorType for I2CIf {
    type Error = I2CIfError;
}

impl I2c for I2CIf {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> std::prelude::v1::Result<(), Self::Error> {
        let ops = operations
            .iter()
            .map(|op| match op {
                embedded_hal::i2c::Operation::Read(items) => I2CIfOps::Read(items.len()),
                embedded_hal::i2c::Operation::Write(items) => {
                    I2CIfOps::Write(Vec::from_iter(items.iter().cloned()))
                }
            })
            .collect();
        let mut resps: Vec<Vec<u8>> = self
            .rw_data(address, ops)
            .await
            .map_err(|_| I2CIfError)?
            .map_err(|_| I2CIfError)?;

        for op in operations {
            if let embedded_hal::i2c::Operation::Read(items) = op {
                items.copy_from_slice(&(resps.pop().ok_or(I2CIfError)?)[..]);
            }
        }
        Ok(())
    }
}

pub struct I2C {}

impl I2C {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for I2C {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let i2c_dev = I2cdev::new("/dev/i2c-0").context("Failed to init i2c")?;
        let i2c_dev = Arc::new(Mutex::new(i2c_dev));

        let i2c_if = I2CIfBackend::builder();
        let i2c_if = i2c_if.on_rw_data(move |addr, ops| {
            let dev = i2c_dev.clone();
            async move {
                let mut bufs: Vec<_> = ops
                    .iter()
                    .filter_map(|op| match op {
                        I2CIfOps::Read(len) => Some(vec![0; *len]),
                        _ => None,
                    })
                    .collect();
                let mut bufs_iter = bufs.iter_mut();
                let mut hal_ops = Vec::new();
                for op in ops.iter() {
                    let hal_op = match op {
                        I2CIfOps::Read(_) => embedded_hal::i2c::Operation::Read(
                            bufs_iter.next().unwrap().as_mut_slice(),
                        ),
                        I2CIfOps::Write(data) => {
                            embedded_hal::i2c::Operation::Write(data.as_slice())
                        }
                    };
                    hal_ops.push(hal_op);
                }
                dev.lock()
                    .await
                    .transaction(addr, &mut hal_ops)
                    .context("Hal error")?;
                Ok(bufs)
            }
        });
        let mut i2c_if = i2c_if.build().context("Failed to build i2c interface")?;

        let i2c_if_front = i2c_if.get_if();
        if_mngr
            .reg("i2c", move || i2c_if_front.clone())
            .await
            .context("Failed to reg i2c interface")?;

        loop {
            i2c_if.poll().await;
        }
    }
}
