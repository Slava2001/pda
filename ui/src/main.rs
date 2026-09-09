#![allow(dead_code)]

mod core;
pub mod modules;
pub mod menu;
pub mod utils;
use core::Core;

#[tokio::main]
async fn main() -> ! {
    let core = Core::new();
    core.run().await
}
