//! # marvis-agent
//!
//! The agent loop and task orchestration layer for Marvis.
//! Implements the core AI ↔ Tool interaction cycle.

pub mod loop_;
pub mod orchestrator;
pub mod planner;

pub use loop_::{AgentConfig, AgentLoop};
pub use orchestrator::TaskOrchestrator;
pub use planner::Planner;
