// src/web.rs
use anyhow::Result;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{get, post, routes, State, fairing::{Fairing, Info, Kind}};
use rocket::http::{Status, Header, ContentType};
use rocket::{Request, Response};
use rocket::response::{self, Responder};
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use std::path::PathBuf;
use std::fs;
use tracing::{info, error, warn};
use crate::{CvConfig, CvGenerator, CvTemplate, list_templates};
use crate::auth::{AuthenticatedUser, OptionalAuth, AuthConfig, AuthError};
use crate::database::{DatabaseConfig, TenantService};

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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub signup_required: Option<bool>,
}

pub struct ServerConfig {
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
    pub templates_dir: PathBuf,
}

// Protected endpoint - requires authentication and tenant validation
#[post("/generate", data = "<request>")]
pub async fn generate_cv(
    request: Json<GenerateRequest>, 
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>
) -> Result<PdfResponse, Status> {
    let user = auth.user();
    let tenant = auth.tenant();
    
    info!("User {} (tenant: {}) generating CV for {}", 
          user.email, tenant.tenant_name, request.person);
    
    let lang = request.lang.as_deref().unwrap_or("en");
    let template_str = request.template.as_deref().unwrap_or("default");
    
    let template = match CvTemplate::from_str(template_str) {
        Ok(t) => t,
        Err(_) => {
            warn!("Invalid template: {}", template_str);
            return Err(Status::BadRequest);
        }
    };

    // Get tenant-specific data directory
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    let tenant_service = TenantService::new(pool);
    let tenant_data_dir = match tenant_service.ensure_tenant_data_dir(&config.data_dir, tenant).await {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to ensure tenant data directory: {}", e);
            return Err(Status::InternalServerError);
        }
    };
    
    let cv_config = CvConfig::new(&request.person, lang)
        .with_template(template)
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());
    
    match CvGenerator::new(cv_config) {
        Ok(generator) => {
            match generator.generate_pdf_data() {
                Ok(pdf_data) => {
                    info!("Successfully generated CV for {} by {} (tenant: {})", 
                          request.person, user.email, tenant.tenant_name);
                    Ok(PdfResponse(pdf_data))
                },
                Err(e) => {
                    error!("Generation error for {} (tenant: {}): {}", 
                           request.person, tenant.tenant_name, e);
                    Err(Status::InternalServerError)
                }
            }
        },
        Err(e) => {
            error!("Config error for {} (tenant: {}): {}", 
                   request.person, tenant.tenant_name, e);
            Err(Status::BadRequest)
        }
    }
}

// Protected endpoint - requires authentication and tenant validation
#[post("/create", data = "<request>")]
pub async fn create_person(
    request: Json<CreatePersonRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>
) -> Result<Json<CreatePersonResponse>, Status> {
    let user = auth.user();
    let tenant = auth.tenant();
    
    info!("User {} (tenant: {}) creating person: {}", 
          user.email, tenant.tenant_name, request.person);

    // Get tenant-specific data directory
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    let tenant_service = TenantService::new(pool);
    let tenant_data_dir = match tenant_service.ensure_tenant_data_dir(&config.data_dir, tenant).await {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to ensure tenant data directory: {}", e);
            return Err(Status::InternalServerError);
        }
    };
    
    let cv_config = CvConfig::new(&request.person, "en")
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());
    
    let generator = CvGenerator { config: cv_config };
    
    match generator.create_person() {
        Ok(_) => {
            let person_dir = generator.config.person_data_dir();
            info!("Person directory created for {} by {} (tenant: {})", 
                  request.person, user.email, tenant.tenant_name);
            
            Ok(Json(CreatePersonResponse {
                success: true,
                message: format!("Person directory created successfully for {}", request.person),
                person_dir: person_dir.to_string_lossy().to_string(),
                created_by: Some(user.email.clone()),
                tenant: tenant.tenant_name.clone(),
            }))
        },
        Err(e) => {
            error!("Person creation error for {} (tenant: {}): {}", 
                   request.person, tenant.tenant_name, e);
            Err(Status::InternalServerError)
        }
    }
}

// Protected endpoint - requires authentication and tenant validation
#[post("/upload-picture", data = "<upload>")]
pub async fn upload_picture(
    mut upload: Form<UploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>
) -> Result<Json<UploadResponse>, Status> {
    let user = auth.user();
    let tenant = auth.tenant();
    
    info!("User {} (tenant: {}) uploading picture for {}", 
          user.email, tenant.tenant_name, upload.person);

    // Get tenant-specific data directory
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    let tenant_service = TenantService::new(pool);
    let tenant_data_dir = match tenant_service.ensure_tenant_data_dir(&config.data_dir, tenant).await {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to ensure tenant data directory: {}", e);
            return Err(Status::InternalServerError);
        }
    };
    
    // Check if person directory exists in tenant's space
    let person_dir = tenant_data_dir.join(&upload.person);
    if !person_dir.exists() {
        return Ok(Json(UploadResponse {
            success: false,
            message: format!("Person directory not found: {}", upload.person),
            file_path: String::new(),
            tenant: tenant.tenant_name.clone(),
        }));
    }
    
    // Validate file type (basic check)
    let content_type = upload.file.content_type();
    let is_image = content_type.map_or(false, |ct| {
        ct.is_png() || ct.is_jpeg() || ct.top() == "image"
    });
    
    if !is_image {
        return Ok(Json(UploadResponse {
            success: false,
            message: "Invalid file type. Please upload an image file (PNG, JPG, etc.)".to_string(),
            file_path: String::new(),
            tenant: tenant.tenant_name.clone(),
        }));
    }
    
    // Save file as profile.png in person's directory
    let target_path = person_dir.join("profile.png");
    
    match upload.file.persist_to(&target_path).await {
        Ok(_) => {
            info!("Profile picture uploaded for {} by {} (tenant: {})", 
                  upload.person, user.email, tenant.tenant_name);
            Ok(Json(UploadResponse {
                success: true,
                message: format!("Profile picture uploaded successfully for {}", upload.person),
                file_path: target_path.to_string_lossy().to_string(),
                tenant: tenant.tenant_name.clone(),
            }))
        },
        Err(e) => {
            error!("File upload error for {} (tenant: {}): {}", 
                   upload.person, tenant.tenant_name, e);
            Err(Status::InternalServerError)
        }
    }
}

// Public endpoint - no authentication required
#[get("/templates")]
pub async fn get_templates(config: &State<ServerConfig>) -> Json<TemplatesResponse> {
    match list_templates(&config.templates_dir) {
        Ok(templates) => {
            let template_infos = templates.into_iter().map(|name| {
                let description = match name.as_str() {
                    "default" => "Standard CV layout",
                    "keyteo" => "CV with Keyteo branding and logo at the top of every page",
                    _ => "Custom template",
                };
                TemplateInfo {
                    name,
                    description: description.to_string(),
                }
            }).collect();

            Json(TemplatesResponse {
                success: true,
                templates: template_infos,
            })
        },
        Err(e) => {
            error!("Failed to list templates: {}", e);
            Json(TemplatesResponse {
                success: false,
                templates: vec![
                    TemplateInfo {
                        name: "default".to_string(),
                        description: "Standard CV layout".to_string(),
                    }
                ],
            })
        }
    }
}

// Test auth endpoint with tenant info
#[get("/me")]
pub async fn get_current_user(auth: AuthenticatedUser) -> Json<AuthResponse> {
    let user = auth.user();
    let tenant = auth.tenant();
    
    Json(AuthResponse {
        success: true,
        user: Some(UserInfo {
            uid: user.uid.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            picture: user.picture.clone(),
            tenant_name: tenant.tenant_name.clone(),
        }),
        message: format!("User authenticated successfully for tenant: {}", tenant.tenant_name),
    })
}

// Handle authentication errors with proper error responses
#[get("/me", rank = 2)]
pub async fn get_current_user_error() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        success: false,
        error: "Authentication required or user not authorized for any tenant".to_string(),
        signup_required: Some(true),
    })
}

// Public health check with optional tenant info
#[get("/health")]
pub async fn health(auth: OptionalAuth) -> Json<&'static str> {
    if let Some(user) = auth.user {
        info!("Health check by authenticated user: {} (tenant: {})", 
              user.user().email, user.tenant().tenant_name);
    } else {
        info!("Health check by anonymous user");
    }
    Json("OK")
}

// Handle OPTIONS requests for CORS preflight
#[rocket::options("/<_..>")]
pub async fn options() -> Status {
    Status::Ok
}

pub async fn start_web_server(
    data_dir: PathBuf, 
    output_dir: PathBuf, 
    templates_dir: PathBuf
) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
    .with_ansi(false)  // Disable ANSI colors
    .with_env_filter("info")
    .init();

    let server_config = ServerConfig {
        data_dir: data_dir.clone(),
        output_dir, 
        templates_dir,
    };

    // Ensure data directory exists BEFORE creating database
    tokio::fs::create_dir_all(&data_dir).await?;

    // Initialize database
    let database_path = data_dir.join("tenants.db");
    let mut db_config = DatabaseConfig::new(database_path);
    
    // Initialize database pool and run migrations
    if let Err(e) = db_config.init_pool().await {
        error!("Failed to initialize database: {}", e);
        return Err(e);
    }
    
    if let Err(e) = db_config.migrate().await {
        error!("Failed to run database migrations: {}", e);
        return Err(e);
    }

    // Initialize auth config with your Firebase project ID
    let mut auth_config = AuthConfig::new("semantic-27923".to_string());
    
    // Fetch Firebase public keys
    if let Err(e) = auth_config.update_firebase_keys().await {
        error!("Failed to fetch Firebase keys: {}", e);
        return Err(e);
    }

    info!("Starting Multi-tenant CV Generator API server");
    info!("Database: {}", db_config.database_path.display());
    info!("Protected endpoints require Firebase Authentication + Tenant Authorization");
    
    let _rocket = rocket::build()
        .attach(Cors)
        .manage(server_config)
        .manage(auth_config)
        .manage(db_config)
        .mount("/api", routes![
            generate_cv, 
            create_person, 
            upload_picture, 
            get_templates, 
            get_current_user,
            get_current_user_error,
            health,
            options
        ])
        .launch()
        .await;

    Ok(())
}
