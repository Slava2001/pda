pub mod manager;
mod apps;

use async_trait::async_trait;
use crate::{display::Display, key_event::KeyEventType};

pub struct AppError;

#[async_trait]
pub trait App: Send + Sync {
    async fn run(&self) -> Result<(), AppError>;
    async fn key_event(&self, event: KeyEventType) -> Result<(), AppError>;
    async fn render(&self, display: &mut Display) -> Result<(), AppError>;
}
