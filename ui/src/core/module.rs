use super::interface::IfMngr;
use anyhow::Result;

#[async_trait::async_trait]
pub trait Module: Send + Sync {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()>;
}
