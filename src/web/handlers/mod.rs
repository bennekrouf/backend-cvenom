// src/web/handlers/mod.rs - Fixed to include upload_picture_handler

pub mod cv_handlers;
pub mod linkedin_handlers;
pub mod person_handlers;
pub mod system_handlers;

pub use cv_handlers::*;
pub use linkedin_handlers::*;
pub use person_handlers::*;
pub use system_handlers::*;

// Explicitly re-export the upload_picture_handler to ensure it's available
pub use person_handlers::upload_picture_handler;

