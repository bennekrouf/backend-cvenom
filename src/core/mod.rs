// src/core/mod.rs
//! Core services to eliminate redundancy while preserving API compatibility

pub mod config_manager;
pub mod database;
pub mod fs_ops;
pub mod service_client;
pub mod template_engine;

pub use config_manager::ConfigManager;
pub use database::Database;
pub use fs_ops::FsOps;
pub use service_client::ServiceClient;
pub use template_engine::TemplateEngine;

