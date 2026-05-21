// src/web/handlers/model_handlers.rs
//! Admin endpoints to read/write the cv-import service's config.yaml
//! and trigger a hot-reload (PM2 restart).

use crate::auth::AuthenticatedUser;
use crate::web::types::StandardErrorResponse;
use graflog::app_log;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

const ADMIN_EMAIL: &str = "mohamed.bennekrouf@gmail.com";

fn admin_only(auth: &AuthenticatedUser) -> Result<(), Json<StandardErrorResponse>> {
    if auth.email().to_lowercase() != ADMIN_EMAIL {
        Err(Json(StandardErrorResponse::new(
            "Admin access required".to_string(),
            "FORBIDDEN".to_string(),
            vec![],
            None,
        )))
    } else {
        Ok(())
    }
}

fn config_path() -> String {
    std::env::var("CV_IMPORT_CONFIG_PATH")
        .unwrap_or_else(|_| "../cv-import/config.yaml".to_string())
}

fn cv_service_url() -> String {
    std::env::var("CV_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:5555".to_string())
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ProviderModelConfig {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    /// Masked API key for display — never return the full key.
    #[serde(skip_deserializing)]
    pub api_key_masked: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ModelConfig {
    pub providers: OperationProviders,
    pub claude: Option<ProviderModelConfig>,
    pub cohere: Option<ProviderModelConfig>,
    pub deepseek: Option<ProviderModelConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct OperationProviders {
    pub cv_import: String,
    pub translation: String,
    pub job_matching: String,
    pub cv_optimization: String,
    pub cover_letter: String,
    pub portfolio: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ModelConfigResponse {
    pub success: bool,
    pub config: ModelConfig,
    pub config_path: String,
}

// ── GET /admin/models ─────────────────────────────────────────────────────────

pub async fn get_model_config_handler(
    auth: AuthenticatedUser,
) -> Result<Json<ModelConfigResponse>, Json<StandardErrorResponse>> {
    admin_only(&auth)?;

    let path = config_path();
    let content = std::fs::read_to_string(&path).map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Cannot read config.yaml at '{}': {}", path, e),
            "CONFIG_READ_ERROR".to_string(),
            vec![format!("Set CV_IMPORT_CONFIG_PATH env var (current: {})", path)],
            None,
        ))
    })?;

    let mut config: ModelConfig = serde_yaml::from_str(&content).map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Cannot parse config.yaml: {}", e),
            "CONFIG_PARSE_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;

    // Read raw YAML to extract api_key fields and mask them for display
    let raw: serde_yaml::Value = serde_yaml::from_str(&content).unwrap_or_default();
    fn mask_key(raw: &serde_yaml::Value, provider: &str) -> Option<String> {
        let key = raw.get(provider)?.get("api_key")?.as_str()?;
        if key.is_empty() { return None; }
        let visible = if key.len() > 8 { &key[..4] } else { "" };
        Some(format!("{}…{}", visible, &key[key.len().saturating_sub(4)..]))
    }
    if let Some(ref mut c) = config.claude {
        c.api_key_masked = mask_key(&raw, "claude");
    }
    if let Some(ref mut c) = config.cohere {
        c.api_key_masked = mask_key(&raw, "cohere");
    }
    if let Some(ref mut c) = config.deepseek {
        c.api_key_masked = mask_key(&raw, "deepseek");
    }

    app_log!(info, admin = %auth.email(), "Model config read from {}", path);
    Ok(Json(ModelConfigResponse { success: true, config, config_path: path }))
}

// ── POST /admin/models ────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateProviderModelConfig {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    /// Optional API key. If provided and non-empty, written to config.yaml.
    /// If omitted or empty, the existing key is preserved (no-op).
    pub api_key: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateModelConfigRequest {
    pub providers: OperationProviders,
    pub claude: Option<UpdateProviderModelConfig>,
    pub cohere: Option<UpdateProviderModelConfig>,
    pub deepseek: Option<UpdateProviderModelConfig>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateModelConfigResponse {
    pub success: bool,
    pub message: String,
    pub restarted: bool,
}

pub async fn update_model_config_handler(
    body: Json<UpdateModelConfigRequest>,
    auth: AuthenticatedUser,
) -> Result<Json<UpdateModelConfigResponse>, Json<StandardErrorResponse>> {
    admin_only(&auth)?;

    let valid_providers = ["claude", "cohere", "deepseek"];
    for (op, prov) in [
        ("cv_import", &body.providers.cv_import),
        ("translation", &body.providers.translation),
        ("job_matching", &body.providers.job_matching),
        ("cv_optimization", &body.providers.cv_optimization),
        ("cover_letter", &body.providers.cover_letter),
        ("portfolio", &body.providers.portfolio),
    ] {
        if !valid_providers.contains(&prov.as_str()) {
            return Err(Json(StandardErrorResponse::new(
                format!("Invalid provider '{}' for operation '{}'", prov, op),
                "INVALID_PROVIDER".to_string(),
                vec![format!("Valid providers: {}", valid_providers.join(", "))],
                None,
            )));
        }
    }

    // Read existing config to preserve comments-free YAML and merge
    let path = config_path();
    let existing_raw = std::fs::read_to_string(&path).unwrap_or_default();
    let mut yaml: serde_yaml::Value =
        serde_yaml::from_str(&existing_raw).unwrap_or(serde_yaml::Value::Mapping(Default::default()));

    // Update providers block
    let providers_map = yaml
        .get_mut("providers")
        .and_then(|v| v.as_mapping_mut())
        .ok_or_else(|| {
            Json(StandardErrorResponse::new(
                "config.yaml missing 'providers' block".to_string(),
                "CONFIG_INVALID".to_string(),
                vec![],
                None,
            ))
        })?;

    for (key, value) in [
        ("cv_import", &body.providers.cv_import),
        ("translation", &body.providers.translation),
        ("job_matching", &body.providers.job_matching),
        ("cv_optimization", &body.providers.cv_optimization),
        ("cover_letter", &body.providers.cover_letter),
        ("portfolio", &body.providers.portfolio),
    ] {
        providers_map.insert(
            serde_yaml::Value::String(key.to_string()),
            serde_yaml::Value::String(value.clone()),
        );
    }

    // Update provider model blocks if supplied
    let update_provider = |yaml: &mut serde_yaml::Value, name: &str, cfg: &UpdateProviderModelConfig| {
        if let Some(block) = yaml.get_mut(name).and_then(|v| v.as_mapping_mut()) {
            block.insert(
                serde_yaml::Value::String("model".to_string()),
                serde_yaml::Value::String(cfg.model.clone()),
            );
            block.insert(
                serde_yaml::Value::String("max_tokens".to_string()),
                serde_yaml::Value::Number(cfg.max_tokens.into()),
            );
            block.insert(
                serde_yaml::Value::String("temperature".to_string()),
                serde_yaml::Value::Number(
                    serde_yaml::Number::from(cfg.temperature),
                ),
            );
            // Write API key only if a new non-empty value was provided
            if let Some(ref key) = cfg.api_key {
                if !key.is_empty() {
                    block.insert(
                        serde_yaml::Value::String("api_key".to_string()),
                        serde_yaml::Value::String(key.clone()),
                    );
                }
            }
        }
    };

    if let Some(ref cfg) = body.claude {
        update_provider(&mut yaml, "claude", cfg);
    }
    if let Some(ref cfg) = body.cohere {
        update_provider(&mut yaml, "cohere", cfg);
    }
    if let Some(ref cfg) = body.deepseek {
        update_provider(&mut yaml, "deepseek", cfg);
    }

    let new_content = serde_yaml::to_string(&yaml).map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Failed to serialise config: {}", e),
            "CONFIG_SERIALISE_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;

    std::fs::write(&path, &new_content).map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Failed to write config.yaml: {}", e),
            "CONFIG_WRITE_ERROR".to_string(),
            vec![format!("Check write permissions on {}", path)],
            None,
        ))
    })?;

    app_log!(info, admin = %auth.email(), path = %path, "Model config updated");

    // Signal cv-import to reload (POST /admin/reload → process exits, PM2 restarts it)
    let reload_url = format!("{}/admin/reload", cv_service_url());
    let restarted = match reqwest::Client::new().post(&reload_url).send().await {
        Ok(resp) => {
            app_log!(info, status = %resp.status(), "cv-import reload signalled");
            true
        }
        Err(e) => {
            app_log!(warn, error = %e, "cv-import reload signal failed — restart manually");
            false
        }
    };

    Ok(Json(UpdateModelConfigResponse {
        success: true,
        message: if restarted {
            "Config saved and cv-import service restarting (~2s)".to_string()
        } else {
            "Config saved — restart cv-import manually for changes to take effect".to_string()
        },
        restarted,
    }))
}
