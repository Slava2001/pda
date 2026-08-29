use crate::{
    core::{interface::IfMngr, mod_set::ModSet},
    create_imc_interface,
    modules::init::Init,
};
use tokio::select;
use anyhow::Result;
use std::sync::Arc;

pub mod interface;
pub mod mid;
mod mod_set;
pub mod module;

use mid::Mid;
use module::Module;

#[derive(Clone)]
pub enum CoreEvent {
    ExitOk(Mid),
    ExitErr(Mid),
    Canceled(Mid)
}

create_imc_interface! {
    pub interface CoreIf {
        fn run(module: Box<dyn Module>) -> Result<Mid>;
        fn kill(mid: Mid) -> Result<()>;
        fn subscribe() -> tokio::sync::broadcast::Receiver<CoreEvent>;
    }
}

pub struct Core {}

impl Core {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(self) -> ! {
        let mut mods = ModSet::new();
        let if_mngr = IfMngr::new();

        let mut interface = CoreIfBackend::builder()
            .build()
            .expect("Failed to build core interface backend");

        let core_if = interface.get_if();
        if_mngr
            .reg("core", move || core_if.clone())
            .await
            .expect("Failed to reg core interface");

        let init_module = Box::new(
            Init::build(if_mngr.clone())
                .await
                .expect("Failed to build init module"),
        );

        mods.add(if_mngr.clone(), init_module as Box<dyn Module>)
            .expect("Failed to run init module");

        let event_tx = Arc::new(tokio::sync::broadcast::channel(10).0);

        loop {
            select! {
                Some(event) = mods.poll() => { event_tx.send(event).ok(); }
                event = interface.poll_event() => {
                    match event {
                        CoreIfEvent::run(((module,), rc)) => {
                            let result = mods.add(if_mngr.clone(), module);
                            rc.send(result).ok();
                        },
                        CoreIfEvent::kill(((mid,), rc)) => {
                            let result = mods.kill(mid);
                            rc.send(result).ok();
                        }
                        CoreIfEvent::subscribe(((), rc)) => {
                            rc.send(event_tx.subscribe()).ok();
                        }
                    }
                }
            }
        }
    }
}
