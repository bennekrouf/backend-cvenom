// src/web/handlers/cv_handlers/mod.rs
//! CV handlers module - refactored into separate files for better maintainability

pub mod generate;
pub mod helpers;
pub mod optimize;
pub mod translate;
pub mod upload_convert;

// Re-export all handler functions
pub use generate::generate_cv_handler;
pub use optimize::optimize_cv_handler;
pub use translate::translate_cv_handler;
pub use upload_convert::upload_and_convert_cv_handler;

// Re-export helper functions for use in other modules
pub use helpers::{create_person_from_cv_data, load_person_cv_data, normalize_template};
