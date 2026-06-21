//! # marvis-security
//!
//! Security and permission management for Marvis.
//! Handles permission checks, sensitive operation confirmation,
//! and sandbox isolation for tool execution.

pub mod permissions;
pub mod sandbox;

pub use permissions::SecurityManager;
pub use sandbox::Sandbox;
