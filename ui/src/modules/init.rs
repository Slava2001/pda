use std::time::Duration;

use crate::{
    core::{CoreIf, interface::IfMngr, module::Module},
    modules::build_module
};
use anyhow::Result;
use async_trait::async_trait;
use tokio::time::sleep;

pub struct Init {}

impl Init {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for Init {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let mut core_if: CoreIf = if_mngr.get("core").await?;

        let modules = &[
            ("I2C", None),
            ("NetCtrl", Some(1)),
            ("System", None),
            ("MeteoSensor", None),
            ("PowerCtrl", None),
            ("Display", None),
            ("Keyboard", Some(1)),
            ("Desktop", None),
        ];

        for (module, delay) in modules {
            let module = build_module(module, if_mngr.clone()).await?;
            core_if.run(module).await??;
            if let Some(delay) = delay {
                sleep(Duration::from_secs(*delay)).await;
            }
        }
        Ok(())
    }
}
