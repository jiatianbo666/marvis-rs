//! # marvis-core
//!
//! Core types, traits, and error definitions for the Marvis AI assistant.
//! This crate is the foundation that all other crates build upon.

pub mod error;
pub mod tool;
pub mod types;

// Re-export commonly used items
pub use error::MarvisError;
pub use tool::{RiskLevel, Tool};
pub use types::*;
