use crate::core::database::{DatabaseConfig, Tenant, TenantRepository, TenantService};
// src/auth.rs
use crate::web::ServerConfig;
use anyhow::Result;
use graflog::app_log;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct FirebaseUser {
    pub uid: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String, // Firebase project ID
    pub iss: String, // Firebase issuer
    pub sub: String, // User ID (uid)
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
    pub exp: usize, // Expiration timestamp
    pub iat: usize, // Issued at timestamp
}

impl From<Claims> for FirebaseUser {
    fn from(claims: Claims) -> Self {
        Self {
            uid: claims.sub,
            email: claims.email,
            name: claims.name,
            picture: claims.picture,
            email_verified: claims.email_verified,
        }
    }
}

pub struct AuthConfig {
    pub project_id: String,
    /// kid → PEM public key. Wrapped in Arc<RwLock> so we can refresh
    /// in-place after Google rotates keys (typically every ~6 hours).
    pub firebase_keys: Arc<RwLock<HashMap<String, String>>>,
}

impl AuthConfig {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            firebase_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Fetch Firebase public keys and update the cache.
    /// Takes &self so it can be called on a shared reference (e.g. from Rocket State).
    pub async fn update_firebase_keys(&self) -> Result<()> {
        let url = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

        // Force IPv4 — Google blocks OVH IPv6 ranges with 403
        let client = reqwest::Client::builder()
            .local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
            .build()?;

        let response = client.get(url).send().await?;
        let keys: HashMap<String, String> = response.json().await?;

        let mut cache = self.firebase_keys.write().await;
        *cache = keys;
        app_log!(info, "Updated Firebase public keys via IPv4 ({} keys)", cache.len());

        Ok(())
    }
}

/// Authenticated user with tenant information
pub struct AuthenticatedUser {
    pub firebase_user: FirebaseUser,
    pub tenant: Tenant,
}

impl AuthenticatedUser {
    pub fn user(&self) -> &FirebaseUser {
        &self.firebase_user
    }

    pub fn tenant(&self) -> &Tenant {
        &self.tenant
    }

    pub fn email(&self) -> &str {
        &self.firebase_user.email
    }

    pub fn tenant_name(&self) -> &str {
        &self.tenant.tenant_name
    }

    pub async fn ensure_tenant_exists(
        &self,
        config: &ServerConfig,
        db_config: &DatabaseConfig,
    ) -> Result<(), anyhow::Error> {
        let pool = db_config.pool()?;
        let tenant_service = TenantService::new(pool);

        // Only ensure tenant directory exists, don't create default files
        tenant_service
            .ensure_tenant_data_dir(&config.data_dir, &self.tenant)
            .await?;

        Ok(())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_config = match req.guard::<&State<AuthConfig>>().await {
            Outcome::Success(config) => config,
            Outcome::Error((status, _)) => {
                return Outcome::Error((status, AuthError::DatabaseError))
            }
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        let db_config = match req.guard::<&State<DatabaseConfig>>().await {
            Outcome::Success(config) => config,
            Outcome::Error((status, _)) => {
                return Outcome::Error((status, AuthError::DatabaseError))
            }
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        // Extract Authorization header
        let token = match req.headers().get_one("Authorization") {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            Some(_) => {
                app_log!(warn, "Invalid Authorization header format");
                return Outcome::Error((Status::Unauthorized, AuthError::InvalidToken));
            }
            None => {
                app_log!(warn, "Missing Authorization header");
                return Outcome::Error((Status::Unauthorized, AuthError::MissingToken));
            }
        };

        // Verify the Firebase ID token
        let firebase_user = match verify_firebase_token(token, auth_config).await {
            Ok(user) => user,
            Err(e) => {
                app_log!(error, "Token verification failed: {}", e);
                return Outcome::Error((Status::Unauthorized, AuthError::TokenVerificationFailed));
            }
        };

        // Check tenant access
        let pool = match db_config.pool() {
            Ok(pool) => pool,
            Err(e) => {
                app_log!(error, "Database connection failed: {}", e);
                return Outcome::Error((Status::InternalServerError, AuthError::DatabaseError));
            }
        };

        let tenant_service = TenantService::new(pool);

        // Try to get existing tenant, or create new one
        let (tenant, is_new_user) = match tenant_service
            .get_or_create_tenant(&firebase_user.email)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                app_log!(
                    error,
                    "Failed to get or create tenant for {}: {}",
                    firebase_user.email,
                    e
                );
                return Outcome::Error((Status::InternalServerError, AuthError::DatabaseError));
            }
        };

        // Grant free-offer welcome credits to brand-new users (fire-and-forget)
        // Free offer: 200 credits → 10 CV generations at 20 credits each
        if is_new_user {
            let email_clone = firebase_user.email.clone();
            tokio::spawn(async move {
                const WELCOME_CREDITS: i64 = 200;
                if let (Ok(store_url), Ok(secret)) = (
                    std::env::var("API0_STORE_URL"),
                    std::env::var("API0_INTERNAL_SECRET"),
                ) {
                    let client = reqwest::Client::new();
                    let body = serde_json::json!({ "email": email_clone, "amount": WELCOME_CREDITS });
                    match client
                        .post(format!("{}/api/user/credits", store_url))
                        .header("Content-Type", "application/json")
                        .header("X-Internal-Secret", secret)
                        .json(&body)
                        .send()
                        .await
                    {
                        Ok(_) => app_log!(
                            info,
                            "Granted {} welcome credits to new user: {}",
                            WELCOME_CREDITS,
                            email_clone
                        ),
                        Err(e) => app_log!(
                            error,
                            "Failed to grant welcome credits to {}: {}",
                            email_clone,
                            e
                        ),
                    }
                }
            });
        }

        // Fire-and-forget: update last_seen_at so the retention cleanup knows this user is active.
        if let Ok(pool) = db_config.pool() {
            let touch_pool = pool.clone();
            let touch_email = firebase_user.email.clone();
            tokio::spawn(async move {
                let repo = TenantRepository::new(&touch_pool);
                let _ = repo.touch_last_seen(&touch_email).await;
            });
        }

        app_log!(
            info,
            "User {} authenticated for tenant: {}",
            firebase_user.email,
            tenant.tenant_name
        );

        Outcome::Success(AuthenticatedUser {
            firebase_user,
            tenant,
        })
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    TokenVerificationFailed,
    NotAuthorized,
    DatabaseError,
    SignupRequired,
}

impl AuthError {
    pub fn message(&self) -> &'static str {
        match self {
            AuthError::MissingToken => "Authorization token required",
            AuthError::InvalidToken => "Invalid authorization token format",
            AuthError::TokenVerificationFailed => "Token verification failed",
            AuthError::NotAuthorized => "User not authorized for this tenant. Signup coming soon!",
            AuthError::DatabaseError => "Database error occurred",
            AuthError::SignupRequired => "Signup required. Coming soon!",
        }
    }
}

async fn verify_firebase_token(token: &str, auth_config: &AuthConfig) -> Result<FirebaseUser> {
    // Decode header to get the key ID
    let header = jsonwebtoken::decode_header(token)?;
    let kid = header
        .kid
        .ok_or_else(|| anyhow::anyhow!("Missing kid in token header"))?;

    // Try the cached keys first
    let public_key_opt = {
        let keys = auth_config.firebase_keys.read().await;
        keys.get(&kid).cloned()
    };

    // If kid not found, Google may have rotated keys — refresh and retry once
    let public_key = match public_key_opt {
        Some(k) => k,
        None => {
            app_log!(
                info,
                "Firebase key ID '{}' not in cache — refreshing public keys",
                kid
            );
            auth_config
                .update_firebase_keys()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to refresh Firebase keys: {}", e))?;

            let keys = auth_config.firebase_keys.read().await;
            keys.get(&kid)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Unknown key ID '{}' even after refresh", kid))?
        }
    };

    // Verify the token
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&auth_config.project_id]);
    validation.set_issuer(&[format!(
        "https://securetoken.google.com/{}",
        auth_config.project_id
    )]);

    let decoding_key = DecodingKey::from_rsa_pem(public_key.as_bytes())?;
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    Ok(token_data.claims.into())
}

// Optional auth guard that doesn't fail if no auth is provided
pub struct OptionalAuth {
    pub user: Option<AuthenticatedUser>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalAuth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthenticatedUser::from_request(req).await {
            Outcome::Success(auth) => Outcome::Success(OptionalAuth { user: Some(auth) }),
            _ => Outcome::Success(OptionalAuth { user: None }),
        }
    }
}

/// Legacy FirebaseAuth for backward compatibility (if needed)
pub struct FirebaseAuth {
    user: FirebaseUser,
}

impl FirebaseAuth {
    pub fn user(&self) -> &FirebaseUser {
        &self.user
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for FirebaseAuth {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthenticatedUser::from_request(req).await {
            Outcome::Success(auth) => Outcome::Success(FirebaseAuth {
                user: auth.firebase_user,
            }),
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(f) => Outcome::Forward(f),
        }
    }
}
