use anyhow::Result;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{get, post, routes, State, fairing::{Fairing, Info, Kind}};
use rocket::http::{Status, Header, ContentType};
use rocket::{Request, Response};
use rocket::response::{self, Responder};
use std::path::PathBuf;
use std::fs;
use crate::{CvConfig, CvGenerator};

pub struct PdfResponse(Vec<u8>);

impl<'r> Responder<'r, 'static> for PdfResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::PDF)
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
    }
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GenerateRequest {
    pub person: String,
    pub lang: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GenerateResponse {
    pub success: bool,
    pub message: String,
    pub pdf_path: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

pub struct ServerConfig {
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
    pub templates_dir: PathBuf,
}

#[post("/generate", data = "<request>")]
pub async fn generate_cv(
    request: Json<GenerateRequest>, 
    config: &State<ServerConfig>
) -> Result<PdfResponse, Status> {
    let lang = request.lang.as_deref().unwrap_or("en");
    
    let cv_config = CvConfig::new(&request.person, lang)
        .with_data_dir(config.data_dir.clone())
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());
    
    match CvGenerator::new(cv_config) {
        Ok(generator) => {
            match generator.generate() {
                Ok(pdf_path) => {
                    match fs::read(&pdf_path) {
                        Ok(pdf_data) => {
                            // Clean up the generated file after reading
                            let _ = fs::remove_file(&pdf_path);
                            Ok(PdfResponse(pdf_data))
                        },
                        Err(e) => {
                            eprintln!("Failed to read PDF file: {}", e);
                            Err(Status::InternalServerError)
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Generation error: {}", e);
                    Err(Status::InternalServerError)
                }
            }
        },
        Err(e) => {
            eprintln!("Config error: {}", e);
            Err(Status::BadRequest)
        }
    }
}

#[get("/health")]
pub async fn health() -> Json<&'static str> {
    Json("OK")
}

pub async fn start_web_server(
    data_dir: PathBuf, 
    output_dir: PathBuf, 
    templates_dir: PathBuf
) -> Result<()> {
    let server_config = ServerConfig {
        data_dir,
        output_dir, 
        templates_dir,
    };

    let _rocket = rocket::build()
        .attach(Cors)
        .manage(server_config)
        .mount("/api", routes![generate_cv, health])
        .launch()
        .await;

    Ok(())
}
