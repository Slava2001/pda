use std::{fmt::Display, format};

#[derive(Debug)]
pub struct Error {
    err: String,
}

impl<T: Display> From<T> for Error {
    fn from(value: T) -> Self {
        Self { err: format!("{value}") }
    }
}

pub trait Context<V> {
    fn cc(self, err: &str) -> Result<V, Error>;
}

impl<V, E> Context<V> for Result<V, E> {
    fn cc(self, err: &str) -> Result<V, Error> {
        self.map_err(|_| Error {
            err: format!("Failed to {err}"),
        })
    }
}
