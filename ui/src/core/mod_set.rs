use crate::core::{
    interface::IfMngr,
    mid::{Mid, MidGenerator},
    module::Module,
};
use anyhow::{Result, ensure};
use std::{collections::HashMap, format, ops::Not, println};
use tokio::task::{AbortHandle, JoinSet};

pub struct ModSet {
    runs: JoinSet<Result<()>>,
    mid_map: HashMap<Mid, AbortHandle>,
    mid_gen: MidGenerator,
}

impl ModSet {
    pub fn new() -> Self {
        Self {
            runs: JoinSet::new(),
            mid_map: HashMap::new(),
            mid_gen: MidGenerator::new(),
        }
    }

    pub fn add(&mut self, if_mngr: IfMngr, mut module: Box<dyn Module>) -> Result<Mid> {
        let handle = self.runs.spawn(async move { module.run(if_mngr).await });
        let mid = self.mid_gen.next();
        ensure!(
            self.mid_map.contains_key(&mid).not(),
            "Failed to insert mid"
        );
        self.mid_map.insert(mid, handle);
        println!("Module: {mid}, started");
        Ok(mid)
    }

    pub async fn poll(&mut self) -> Result<()> {
        let (id, event) = match self.runs.join_next_with_id().await {
            Some(Ok((id, Ok(())))) => (id, format!("finished")),
            Some(Ok((id, err))) => (id, format!("return error: {err:?}")),
            Some(Err(err)) => (err.id(), format!("panicked or canceled")),
            None => return Ok(()),
        };

        self.mid_map.retain(|mid, v| {
            if v.id() == id {
                println!("Module: {mid}, {event}");
            }
            v.id() == id
        });
        Ok(())
    }
}
