use std::sync::Arc;

use crate::app::{App, apps::{clock::Clock, radio::Radio}};

pub mod clock;
pub mod radio;

#[derive(Clone, Copy)]
pub enum AppCtlAppKind {
    Clock,
    Radio
}

impl AsRef<str> for AppCtlAppKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Clock => "Clock",
            Self::Radio => "Radio",
        }
    }
}

pub fn build_app(kind: AppCtlAppKind) -> Arc<dyn App> {
    match kind {
        AppCtlAppKind::Clock => Arc::new(Clock::new()) as Arc<dyn App>,
        AppCtlAppKind::Radio => Arc::new(Radio::new()) as Arc<dyn App>,
    }
}
