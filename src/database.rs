// src/database.rs
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tracing::info;

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
        println!(
            "Attempting to create database at: {}",
            self.database_path.display()
        );

        // Ensure parent directory exists
        if let Some(parent) = self.database_path.parent() {
            println!("Creating parent directory: {}", parent.display());
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }

        let database_url = format!("sqlite:{}?mode=rwc", self.database_path.display());
        println!("Database URL: {}", database_url);

        let pool = SqlitePool::connect(&database_url)
            .await
            .context("Failed to connect to SQLite database")?;
        self.pool = Some(pool);

        info!("Database connection pool initialized: {}", database_url);
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

        info!("Database migrations completed successfully");
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

        info!("Created email tenant: {} for email: {}", tenant_name, email);
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

        info!(
            "Created domain tenant: {} for domain: {}",
            tenant_name, domain
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
            info!("Deactivated tenant for email: {}", email);
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
            info!("Deactivated tenant for domain: {}", domain);
        }

        Ok(updated)
    }
}

/// Utility functions for tenant management
pub struct TenantService<'a> {
    repo: TenantRepository<'a>,
}

// Update TenantService in src/database.rs
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
                    info!(
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
                    info!(
                        "User {} failed authorization check for tenant: {}",
                        email, tenant.tenant_name
                    );
                    Ok(None)
                }
            }
            None => {
                info!(
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
            info!("Created tenant data directory: {}", tenant_dir.display());
        }

        Ok(tenant_dir)
    }
}
