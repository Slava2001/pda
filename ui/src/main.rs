#![allow(dead_code)]

mod core;
pub mod modules;
use core::Core;

#[tokio::main]
async fn main() -> ! {
    let core = Core::new();
    core.run().await
}
