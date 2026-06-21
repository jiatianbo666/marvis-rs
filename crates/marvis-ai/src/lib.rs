//! # marvis-ai
//!
//! AI provider abstraction layer for Marvis.
//! Supports multiple AI backends through a common `AiClient` trait.

pub mod client;
pub mod types;

pub mod deepseek;
pub mod qwen;

pub mod mock;

pub use client::AiClient;
pub use types::*;
