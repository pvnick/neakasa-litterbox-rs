//! Async Neakasa client, translated from the Python SDK at 4be4fc7.
//! Credentials and persisted sessions are compatible with the local Python wrapper.
pub mod auth;
pub mod client;
mod constants;
pub mod error;
pub mod json;
pub mod models;
pub mod mqtt;
pub mod state;
pub mod stream;
pub mod transport;
pub use client::NeakasaClient;
pub use error::{Error, Result};
pub use models::*;
pub use stream::{ReconnectPolicy, StatusStream, StreamOptions};

#[cfg(test)]
#[path = "../tests/aligned/mod.rs"]
mod aligned;
