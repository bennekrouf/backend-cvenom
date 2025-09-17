// src/web/handlers/mod.rs
pub mod cv_handlers;
pub mod person_handlers;
pub mod system_handlers;

// Re-export all handlers for easy importing
pub use cv_handlers::*;
pub use person_handlers::*;
pub use system_handlers::*;

pub mod linkedin_handlers;
pub use linkedin_handlers::*;
