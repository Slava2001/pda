use crate::{
    core::{CoreEvent, CoreIf, interface::IfMngr, module::Module},
    menu::entries::{AlwaysFocused, MenuEntry, list::List},
    modules::{
        build_module,
        display::DisplayIf,
        keyboard::{Key, KeyEvent, KeyboardIf},
    },
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::{
    select,
    sync::{
        broadcast::{Receiver, error::RecvError},
        mpsc,
    },
};

pub struct Desktop {
    keys: Receiver<KeyEvent>,
    display: DisplayIf,
    core: CoreIf,
}

impl Desktop {
    pub async fn build(if_mngr: IfMngr) -> Result<Self> {
        let keys = if_mngr
            .get::<&str, KeyboardIf>("keyboard")
            .await
            .context("Failed to get keyboard interface")?
            .subscribe()
            .await
            .context("Failed to subscribe to keyboard events")?;
        let display = if_mngr
            .get("display")
            .await
            .context("Failed to get display interface")?;
        let core = if_mngr
            .get("core")
            .await
            .context("Failed to get core interface")?;
        Ok(Self {
            keys,
            display,
            core,
        })
    }
}

#[async_trait]
impl Module for Desktop {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let (tx, mut list_channel) = mpsc::channel(10);
        let mut menu = List::new("", vec!["CommonTest", "ExitTest"], move |v| {
            tx.try_send(*v).ok();
        });
        menu.set_window(self.display.text_mode_size().await?.1);

        let mut current_mid = None;
        let mut core_events = self.core.subscribe().await?;

        self.display.clear().await?;
        menu.render(&mut self.display).await;
        self.display.flush().await?;
        loop {
            select! {
                res = self.keys.recv() => {
                    let key = match res {
                        Ok(key) => key,
                        Err(RecvError::Lagged(_)) => continue,
                        err => err.context("Failed to receive key event")?
                    };

                    if let Some(mid) = current_mid {
                        if let KeyEvent::Press(Key::VolumeUp) = key {
                            self.core.kill(mid).await?.ok();
                        }
                    } else {
                        menu.update(&mut AlwaysFocused, key).await;
                        menu.render(&mut self.display).await;
                        self.display.flush().await?;
                    }
                }
                Some(mod_type) = list_channel.recv() => {
                    self.display.clear().await?;
                    self.display.flush().await?;
                    let module = build_module(mod_type, if_mngr.clone()).await?;
                    current_mid = Some(self.core.run(module).await??);
                }
                res = core_events.recv() => {
                    let event = match res {
                        Ok(event) => event,
                        Err(RecvError::Lagged(_)) => continue,
                        err => err.context("Failed to receive key event")?
                    };
                    let mid = match event {
                        CoreEvent::ExitOk(mid) => mid,
                        CoreEvent::ExitErr(mid) => mid,
                        CoreEvent::Canceled(mid) => mid,
                    };
                    if current_mid == Some(mid) {
                        current_mid = None;
                        self.display.clear().await?;
                        menu.render(&mut self.display).await;
                        self.display.flush().await?;
                    }
                }
            }
        }
    }
}
