#![allow(dead_code)]

use crate::app::manager::Manager;

mod display;
mod key_event;
mod app;
mod menu;
mod si4732;

#[tokio::main]
async fn main() -> ! {
    let app_manager = Manager::new();
    app_manager.run().await
}
