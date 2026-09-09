use crate::core::{
    CoreEvent,
    interface::IfMngr,
    mid::{Mid, MidGenerator},
    module::Module,
};
use anyhow::{Context, Error, Result, bail, ensure};
use std::{collections::HashMap, fmt::Display, ops::Not, println};
use tokio::task::{AbortHandle, JoinSet};

#[derive(Debug)]
enum ModSetEvent {
    ExitOk,
    ExitErr(Error),
    Canceled,
}

impl Display for ModSetEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModSetEvent::ExitOk => write!(f, "ExitOk"),
            ModSetEvent::ExitErr(error) => write!(
                f,
                "ExitError: \n{}",
                error
                    .chain().enumerate()
                    .map(|(i, e)| format!("{i}) {e}\n"))
                    .collect::<String>()
            ),
            ModSetEvent::Canceled => write!(f, "Canceled"),
        }
    }
}

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
        let mod_name = module
            .name()
            .rsplit("::")
            .next()
            .context("Invalid module name")?;
        let handle = self.runs.spawn(async move { module.run(if_mngr).await });
        let mid = self.mid_gen.next();
        ensure!(
            self.mid_map.contains_key(&mid).not(),
            "Failed to insert mid"
        );
        self.mid_map.insert(mid, handle);
        println!("Module: {mod_name}, mid: {mid}, started");
        Ok(mid)
    }

    pub fn kill(&mut self, mid: Mid) -> Result<()> {
        let Some(handle) = self.mid_map.get(&mid) else {
            bail!(Error::msg("module not found"));
        };
        handle.abort();
        Ok(())
    }

    pub async fn poll(&mut self) -> Option<CoreEvent> {
        let (id, event) = match self.runs.join_next_with_id().await {
            Some(Ok((id, Ok(())))) => (id, ModSetEvent::ExitOk),
            Some(Ok((id, Err(err)))) => (id, ModSetEvent::ExitErr(err)),
            Some(Err(err)) => (err.id(), ModSetEvent::Canceled),
            None => return None,
        };

        let mut found_mid = None;
        self.mid_map.retain(|mid, v| {
            if v.id() == id {
                found_mid = Some(*mid);
                println!("Module: {mid}, {event}");
            }
            v.id() != id
        });
        let mid = found_mid.expect("Unexpected error: module not found: tid: {id}");
        Some(match event {
            ModSetEvent::ExitOk => CoreEvent::ExitOk(mid),
            ModSetEvent::ExitErr(_) => CoreEvent::ExitErr(mid),
            ModSetEvent::Canceled => CoreEvent::Canceled(mid),
        })
    }
}
