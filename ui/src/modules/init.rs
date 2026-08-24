use crate::{
    core::{CoreIf, interface::IfMngr, module::Module},
    modules::{display::Display, keyboard::Keyboard, keyboard_test::KeyboardTest},
};
use anyhow::Result;
use async_trait::async_trait;

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

        let module = Display::build(if_mngr.clone()).await?;
        core_if.run(Box::new(module)).await??;

        let module = Keyboard::build(if_mngr.clone()).await?;
        core_if.run(Box::new(module)).await??;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let module = KeyboardTest::build(if_mngr.clone()).await?;
        core_if.run(Box::new(module)).await??;
        Ok(())
    }
}
