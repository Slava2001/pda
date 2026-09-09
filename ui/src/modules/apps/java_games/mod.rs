mod emulator;
mod game;
use crate::{
    core::{interface::IfMngr, module::Module},
    menu::entries::{AlwaysFocused, MenuEntry, list::List},
    modules::{
        apps::java_games::{emulator::Emulator, game::load_games},
        display::DisplayIf,
        keyboard::{KeyEvent, KeyboardIf},
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

pub struct JavaGames {
    keys: Receiver<KeyEvent>,
    display: DisplayIf,
}

impl JavaGames {
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
        Ok(Self { keys, display })
    }
}

#[async_trait]
impl Module for JavaGames {
    async fn run(&mut self, _if_mngr: IfMngr) -> Result<()> {
        self.display.draw_line("Java Games".to_string(), 0).await?;
        self.display.draw_line("Loading...".to_string(), 1).await?;
        self.display.flush().await?;

        let games = load_games("/root/games")?;
        let (tx, mut list_channel) = mpsc::channel(10);
        let mut menu = List::new("", games, move |v| {
            tx.try_send(v.clone()).ok();
        });
        menu.set_window(self.display.text_mode_size().await?.1);

        loop {
            menu.render(&mut self.display).await;
            if let Some(icon) = menu.current_val().icon.as_ref() {
                self.display
                    .draw_img((240 - icon.1) as i32, 0, icon.1, icon.0.clone())
                    .await?;
            }
            self.display.flush().await?;
            self.display.clear().await?;

            select! {
                res = self.keys.recv() => {
                    let key = match res {
                        Ok(key) => key,
                        Err(RecvError::Lagged(_)) => continue,
                        err => err.context("Failed to receive key event")?
                    };
                    menu.update(&mut AlwaysFocused, key).await;
                }
                Some(game) = list_channel.recv() => {
                    self.display.clear().await?;
                    self.display.draw_line("Loading...".to_string(), 0).await?;
                    self.display.draw_line(game.name.clone(), 1).await?;
                    self.display.draw_line(game.path.clone(), 2).await?;
                    if let Some(icon) = game.icon.as_ref() {
                        self.display.draw_img(
                            120 - icon.1 as i32 / 2, 100,
                            icon.1, icon.0.clone()
                        ).await?;
                    }
                    self.display.flush().await?;
                    Emulator::new(&game.path).await?.poll(&mut self.display).await.ok();
                    return Ok(());
                }
            }
        }
    }
}
