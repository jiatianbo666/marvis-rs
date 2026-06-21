//! # marvis-tools
//!
//! Tool system for Marvis. Provides a registry of tools that the AI can invoke
//! to interact with the operating system — file operations, process management,
//! web interaction, clipboard access, and system information.

pub mod clipboard;
pub mod file;
pub mod process;
pub mod registry;
pub mod system;
pub mod web;

pub use registry::ToolRegistry;
