// src/web/types.rs
use rocket::form::FromForm;
use rocket::fs::TempFile;
use rocket::http::ContentType;
use rocket::response::{self, Responder};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, Response};
use std::path::PathBuf;

pub struct PdfResponse(pub Vec<u8>);

impl<'r> Responder<'r, 'static> for PdfResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::PDF)
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
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
