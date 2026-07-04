//! Application composition root.
//!
//! This crate wires application use cases to concrete outbound adapters. UI and
//! transport crates consume the resulting bridge without knowing how the
//! dependencies were constructed.

pub mod bridge;
pub mod config;

pub use bridge::{BootstrapBridge, BootstrapError};
pub use config::BootstrapConfig;
