mod fabric;
pub mod imc_if;
use std::{any::Any, sync::Arc};

use anyhow::Context;
use anyhow::Result;
use fabric::IfFabric;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct IfMngr {
    fabric: Arc<Mutex<IfFabric>>,
}

impl IfMngr {
    pub fn new() -> Self {
        Self {
            fabric: Arc::new(Mutex::new(IfFabric::new())),
        }
    }

    pub async fn reg<
        K: Into<String>,
        I: 'static + Any + Send + Sync,
        F: 'static + Send + Sync + Fn() -> I,
    >(
        &self,
        key: K,
        func: F,
    ) -> Result<()> {
        self.fabric
            .lock()
            .await
            .reg(key.into(), func)
            .context("Failed to reg interface fabric")
    }

    pub async fn get<K: Into<String>, I: 'static + Any>(&self, key: K) -> Result<I> {
        self.fabric
            .lock()
            .await
            .get(key.into())
            .context("Failed to get interface")
    }
}
