// src/web/handlers/cv_handlers/mod.rs
//! CV handlers module - refactored into separate files for better maintainability

pub mod cover_letter;
pub mod cv_data;
pub mod generate;
pub mod helpers;
pub mod optimize;
pub mod save_optimized;
pub mod translate;
pub mod upload_convert;

// Re-export all handler functions
pub use cover_letter::{cover_letter_handler, CoverLetterRequest};
pub use cv_data::{get_cv_data_handler, put_cv_data_handler, CvFormData};
pub use generate::generate_cv_handler;
pub use optimize::{optimize_and_generate_handler, optimize_cv_handler, OptimizeCvRequest};
pub use save_optimized::{save_optimized_handler, SaveOptimizedRequest};
pub use translate::translate_cv_handler;
pub use upload_convert::upload_and_convert_cv_handler;

// Re-export helper functions for use in other modules
pub use helpers::{create_profile_from_cv_data, load_profile_cv_data, normalize_template};
