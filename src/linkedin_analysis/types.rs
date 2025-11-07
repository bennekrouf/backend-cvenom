use serde::{Deserialize, Serialize};

// Internal request format for the job matching API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobMatchApiRequest {
    pub cv_json: String,
    pub job_url: String,
}

// Error response format from the job matching API
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub(crate) struct JobMatchApiError {
//     pub error: String,
// }
