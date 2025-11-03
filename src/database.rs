// src/database.rs
use graflog::app_log;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tenant {
    pub id: i64,
    pub email: Option<String>,
    pub domain: Option<String>,
    pub tenant_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl Tenant {
    /// Check if this tenant authorizes the given email
    pub fn authorizes_email(&self, email: &str) -> bool {
        if let Some(tenant_email) = &self.email {
            // Exact email match
            tenant_email == email
        } else if let Some(domain) = &self.domain {
            // Domain match - extract domain from email
            if let Some(email_domain) = email.split('@').nth(1) {
                domain == email_domain
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub database_path: PathBuf,
    pub pool: Option<SqlitePool>,
}

impl DatabaseConfig {
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            pool: None,
        }
    }

    /// Initialize the database connection pool
    pub async fn init_pool(&mut self) -> Result<()> {
        app_log!(
            info,
            "Attempting to create database at: {}",
            self.database_path.display()
        );

        // Ensure parent directory exists
        if let Some(parent) = self.database_path.parent() {
            app_log!(info, "Creating parent directory: {}", parent.display());
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }

        let database_url = format!("sqlite:{}?mode=rwc", self.database_path.display());
        app_log!(info, "Database URL: {}", database_url);

        let pool = SqlitePool::connect(&database_url)
            .await
            .context("Failed to connect to SQLite database")?;
        self.pool = Some(pool);

        app_log!(
            info,
            "Database connection pool initialized: {}",
            database_url
        );
        Ok(())
    }

    /// Get the database pool
    pub fn pool(&self) -> Result<&SqlitePool> {
        self.pool.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Database pool not initialized. Call init_pool() first.")
        })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        let pool = self.pool()?;

        // Create tenants table with domain support
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT,
            domain TEXT,
            tenant_name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            CONSTRAINT email_or_domain_check CHECK (
                (email IS NOT NULL AND domain IS NULL) OR 
                (email IS NULL AND domain IS NOT NULL)
            )
        );
        "#,
        )
        .execute(pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_email ON tenants(email);")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_domain ON tenants(domain);")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_tenant_name ON tenants(tenant_name);")
            .execute(pool)
            .await?;

        app_log!(info, "Database migrations completed successfully");
        Ok(())
    }
}

pub struct TenantRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> TenantRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Find tenant that authorizes the given email (exact match or domain match)
    pub async fn find_by_email_or_domain(&self, email: &str) -> Result<Option<Tenant>> {
        // Extract domain from email
        let domain = email.split('@').nth(1).unwrap_or("");

        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT id, email, domain, tenant_name, created_at, updated_at, is_active
            FROM tenants 
            WHERE is_active = TRUE AND (
                email = ? OR domain = ?
            )
            ORDER BY email NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(email)
        .bind(domain)
        .fetch_optional(self.pool)
        .await?;

        Ok(tenant)
    }

    /// Create tenant with specific email
    pub async fn create_email_tenant(&self, email: &str, tenant_name: &str) -> Result<Tenant> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO tenants (email, domain, tenant_name, created_at, updated_at, is_active)
            VALUES (?, NULL, ?, ?, ?, TRUE)
            "#,
        )
        .bind(email)
        .bind(tenant_name)
        .bind(now)
        .bind(now)
        .execute(self.pool)
        .await?;

        let tenant_id = result.last_insert_rowid();

        let tenant = Tenant {
            id: tenant_id,
            email: Some(email.to_string()),
            domain: None,
            tenant_name: tenant_name.to_string(),
            created_at: now,
            updated_at: now,
            is_active: true,
        };

        app_log!(
            info,
            "Created email tenant: {} for email: {}",
            tenant_name,
            email
        );
        Ok(tenant)
    }

    /// Create tenant with domain authorization
    pub async fn create_domain_tenant(&self, domain: &str, tenant_name: &str) -> Result<Tenant> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO tenants (email, domain, tenant_name, created_at, updated_at, is_active)
            VALUES (NULL, ?, ?, ?, ?, TRUE)
            "#,
        )
        .bind(domain)
        .bind(tenant_name)
        .bind(now)
        .bind(now)
        .execute(self.pool)
        .await?;

        let tenant_id = result.last_insert_rowid();

        let tenant = Tenant {
            id: tenant_id,
            email: None,
            domain: Some(domain.to_string()),
            tenant_name: tenant_name.to_string(),
            created_at: now,
            updated_at: now,
            is_active: true,
        };

        app_log!(
            info,
            "Created domain tenant: {} for domain: {}",
            tenant_name,
            domain
        );
        Ok(tenant)
    }

    /// List all active tenants
    pub async fn list_active(&self) -> Result<Vec<Tenant>> {
        let tenants = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT id, email, domain, tenant_name, created_at, updated_at, is_active
            FROM tenants 
            WHERE is_active = TRUE
            ORDER BY tenant_name ASC, email ASC, domain ASC
            "#,
        )
        .fetch_all(self.pool)
        .await?;

        Ok(tenants)
    }

    /// Deactivate tenant by email or domain
    pub async fn deactivate_by_email(&self, email: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE tenants 
            SET is_active = FALSE, updated_at = ?
            WHERE email = ?
            "#,
        )
        .bind(Utc::now())
        .bind(email)
        .execute(self.pool)
        .await?;

        let updated = result.rows_affected() > 0;
        if updated {
            app_log!(info, "Deactivated tenant for email: {}", email);
        }

        Ok(updated)
    }

    pub async fn deactivate_by_domain(&self, domain: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE tenants 
            SET is_active = FALSE, updated_at = ?
            WHERE domain = ?
            "#,
        )
        .bind(Utc::now())
        .bind(domain)
        .execute(self.pool)
        .await?;

        let updated = result.rows_affected() > 0;
        if updated {
            app_log!(info, "Deactivated tenant for domain: {}", domain);
        }

        Ok(updated)
    }
}

/// Utility functions for tenant management
pub struct TenantService<'a> {
    repo: TenantRepository<'a>,
}

// Update TenantService in src/database.rs
#[allow(dead_code)]
impl<'a> TenantService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self {
            repo: TenantRepository::new(pool),
        }
    }

    /// Validate user access and get tenant info (now supports domain matching)
    pub async fn validate_user_access(&self, email: &str) -> Result<Option<Tenant>> {
        match self.repo.find_by_email_or_domain(email).await? {
            Some(tenant) => {
                // Double-check authorization using the tenant's logic
                if tenant.authorizes_email(email) {
                    app_log!(
                        info,
                        "User {} validated for tenant: {} ({})",
                        email,
                        tenant.tenant_name,
                        if tenant.email.is_some() {
                            "email"
                        } else {
                            "domain"
                        }
                    );
                    Ok(Some(tenant))
                } else {
                    app_log!(
                        info,
                        "User {} failed authorization check for tenant: {}",
                        email,
                        tenant.tenant_name
                    );
                    Ok(None)
                }
            }
            None => {
                app_log!(
                    info,
                    "Access denied for email: {} - no matching tenant or domain",
                    email
                );
                Ok(None)
            }
        }
    }

    /// Get tenant-specific data directory
    pub fn get_tenant_data_dir(&self, base_data_dir: &PathBuf, tenant: &Tenant) -> PathBuf {
        base_data_dir.join("tenants").join(&tenant.tenant_name)
    }

    /// Ensure tenant data directory exists
    pub async fn ensure_tenant_data_dir(
        &self,
        base_data_dir: &PathBuf,
        tenant: &Tenant,
    ) -> Result<PathBuf> {
        let tenant_dir = self.get_tenant_data_dir(base_data_dir, tenant);

        if !tenant_dir.exists() {
            tokio::fs::create_dir_all(&tenant_dir).await?;
            app_log!(
                info,
                "Created tenant data directory: {}",
                tenant_dir.display()
            );
        }

        Ok(tenant_dir)
    }

    /// Create default person structure for new tenant users
    pub async fn create_default_person(
        &self,
        base_data_dir: &PathBuf,
        templates_dir: &PathBuf,
        tenant: &Tenant,
        person_name: &str,
        display_name: Option<&str>,
    ) -> Result<()> {
        let tenant_data_dir = self.ensure_tenant_data_dir(base_data_dir, tenant).await?;
        let person_dir = tenant_data_dir.join(person_name);

        if person_dir.exists() {
            return Ok(()); // Already exists
        }

        tokio::fs::create_dir_all(&person_dir).await?;

        // Copy default templates
        let person_template = templates_dir.join("person_template.toml");
        let experience_template = templates_dir.join("experiences_template.typ");

        if person_template.exists() {
            let template_content = tokio::fs::read_to_string(&person_template).await?;
            // Use display_name if provided, otherwise use person_name
            let name_for_template = display_name.unwrap_or(person_name);
            let processed = template_content.replace("{{name}}", name_for_template);
            tokio::fs::write(person_dir.join("cv_params.toml"), processed).await?;
        }

        if experience_template.exists() {
            let exp_content = tokio::fs::read_to_string(&experience_template).await?;
            tokio::fs::write(person_dir.join("experiences_en.typ"), &exp_content).await?;
            tokio::fs::write(person_dir.join("experiences_fr.typ"), &exp_content).await?;
        }

        app_log!(
            info,
            "Created default person structure for {} (display: {}) in tenant {}",
            person_name,
            display_name.unwrap_or(person_name),
            tenant.tenant_name
        );
        Ok(())
    }

    /// Auto-create tenant for new user based on email
    pub async fn auto_create_tenant(&self, email: &str) -> Result<Tenant> {
        // Extract username from email (before @)
        let username = email.split('@').next().unwrap_or("user");
        let tenant_name = username.to_string();

        app_log!(
            info,
            "Auto-creating tenant '{}' for new user: {}",
            tenant_name,
            email
        );

        let tenant_repo = TenantRepository::new(self.repo.pool);
        tenant_repo.create_email_tenant(email, &tenant_name).await
    }

    /// Get or create tenant for user
    pub async fn get_or_create_tenant(&self, email: &str) -> Result<Tenant> {
        // First try to find existing tenant
        if let Some(tenant) = self.validate_user_access(email).await? {
            return Ok(tenant);
        }

        // If none found, auto-create
        self.auto_create_tenant(email).await
    }
}

pub fn email_to_folder_name(email: &str) -> String {
    email.replace('@', "-").replace('.', "-")
}

pub fn get_tenant_for_email(email: &str) -> String {
    if let Some(domain) = email.split('@').nth(1) {
        // For known company domains, return the company tenant
        match domain {
            "keyteo.ch" => "keyteo".to_string(),
            // Add other company domains here
            _ => std::env::var("DEFAULT_TENANT").unwrap_or_else(|_| "independent".to_string()),
        }
    } else {
        "independent".to_string()
    }
}

pub fn get_tenant_folder_path(
    email: &str,
    tenant_data_path: &std::path::PathBuf,
) -> std::path::PathBuf {
    let tenant = get_tenant_for_email(email);
    let user_folder = email_to_folder_name(email);

    tenant_data_path.join(tenant).join(user_folder)
}
