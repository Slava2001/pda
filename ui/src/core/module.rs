use std::any::{Any, type_name};

use super::interface::IfMngr;
use anyhow::Result;

#[async_trait::async_trait]
pub trait Module: Any + Send + Sync {
    fn name(&self)-> &'static str {
        type_name::<Self>()
    }

    async fn run(&mut self, if_mngr: IfMngr) -> Result<()>;
}
