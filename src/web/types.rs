// src/web/types.rs - Add missing types and ensure all are properly exported

use rocket::form::FromForm;
use rocket::fs::TempFile;
use rocket::http::ContentType;
use rocket::response::{self, Responder};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, Response};
use std::path::PathBuf;

pub struct PdfResponse {
    pub data: Vec<u8>,
    pub filename: Option<String>,
}

impl PdfResponse {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            filename: None,
        }
    }

    pub fn with_filename(data: Vec<u8>, filename: String) -> Self {
        Self {
            data,
            filename: Some(filename),
        }
    }
}

impl<'r> Responder<'r, 'static> for PdfResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let mut binding = Response::build();
        let mut response = binding
            .header(ContentType::PDF)
            .sized_body(self.data.len(), std::io::Cursor::new(self.data));

        if let Some(filename) = self.filename {
            response = response.raw_header(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            );
        }

        response.ok()
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub error_code: String,
    pub suggestions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RenameCollaboratorRequest {
    pub new_name: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ValidationError {
    pub success: bool,
    pub error: String,
    pub error_code: String,
    pub missing_person: String,
    pub tenant: String,
    pub suggestions: Vec<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ImageValidationErrorResponse {
    pub success: bool,
    pub error: String,
    pub error_code: String,
    pub image_path: String,
    pub suggestions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DeletePersonRequest {
    pub person: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DeletePersonResponse {
    pub success: bool,
    pub message: String,
    pub deleted_person: String,
    pub tenant: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GenerateRequest {
    pub person: String,
    pub lang: Option<String>,
    pub template: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePersonRequest {
    pub person: String,
}

#[derive(FromForm)]
pub struct UploadForm<'f> {
    pub person: String,
    pub file: TempFile<'f>,
}

#[derive(FromForm)]
pub struct CvUploadForm<'f> {
    pub cv_file: TempFile<'f>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePersonResponse {
    pub success: bool,
    pub message: String,
    pub person_dir: String,
    pub created_by: Option<String>,
    pub tenant: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub file_path: String,
    pub tenant: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CvConvertResponse {
    pub success: bool,
    pub message: String,
    pub person_name: String,
    pub tenant: String,
    pub person_dir: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplatesResponse {
    pub success: bool,
    pub templates: Vec<TemplateInfo>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UserInfo {
    pub uid: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub tenant_name: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthResponse {
    pub success: bool,
    pub user: Option<UserInfo>,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SaveFileRequest {
    pub path: String,
    pub content: String,
}

pub struct ServerConfig {
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
    pub templates_dir: PathBuf,
}

// NEW STANDARD RESPONSE TYPES FOR V2 API

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StandardResponse {
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TextResponse {
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DataResponse<T> {
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    pub success: bool,
    pub message: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_format: Option<DisplayFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ActionResponse {
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    pub success: bool,
    pub message: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StandardErrorResponse {
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    pub success: bool,
    pub error: String,
    pub error_code: String,
    pub suggestions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub enum ResponseType {
    Text,
    File,
    Data,
    Action,
    Error,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DisplayFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<DisplaySection>>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DisplaySection {
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<String>>,
}

// Request types with conversation_id support
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct StandardRequest<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

// Helper trait for extracting conversation_id
pub trait WithConversationId {
    fn conversation_id(&self) -> Option<String>;
}

impl<T> WithConversationId for StandardRequest<T> {
    fn conversation_id(&self) -> Option<String> {
        self.conversation_id.clone()
    }
}

// Job analysis response data structure
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct JobAnalysisData {
    pub job_content: Option<crate::linkedin_analysis::JobContent>,
    pub person_experiences: Option<String>,
    pub fit_analysis: Option<String>,
    pub raw_job_content: Option<String>,
}

// Helper functions to create standard responses
impl TextResponse {
    pub fn success(message: String, conversation_id: Option<String>) -> Self {
        Self {
            response_type: ResponseType::Text,
            success: true,
            message,
            conversation_id,
        }
    }
}

impl<T> DataResponse<T> {
    pub fn success(message: String, data: T, conversation_id: Option<String>) -> Self {
        Self {
            response_type: ResponseType::Data,
            success: true,
            message,
            data,
            display_format: None,
            conversation_id,
        }
    }

    pub fn with_display_format(mut self, display_format: DisplayFormat) -> Self {
        self.display_format = Some(display_format);
        self
    }
}

impl ActionResponse {
    pub fn success(message: String, action: String, conversation_id: Option<String>) -> Self {
        Self {
            response_type: ResponseType::Action,
            success: true,
            message,
            action,
            next_actions: None,
            conversation_id,
        }
    }

    pub fn with_next_actions(mut self, next_actions: Vec<String>) -> Self {
        self.next_actions = Some(next_actions);
        self
    }
}

impl StandardErrorResponse {
    pub fn new(
        error: String,
        error_code: String,
        suggestions: Vec<String>,
        conversation_id: Option<String>,
    ) -> Self {
        Self {
            response_type: ResponseType::Error,
            success: false,
            error,
            error_code,
            suggestions,
            conversation_id,
        }
    }
}
