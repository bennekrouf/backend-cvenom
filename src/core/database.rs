// src/core/database.rs
//! Unified database operations - eliminates duplicate connection patterns

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::path::Path;
use graflog::app_log;

use crate::core::FsOps;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create new database connection with automatic setup
    pub async fn new(database_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            FsOps::ensure_dir_exists(parent).await?;
        }

        let database_url = format!("sqlite:{}?mode=rwc", database_path.display());
        let pool = SqlitePool::connect(&database_url).await.with_context(|| {
            format!("Failed to connect to database: {}", database_path.display())
        })?;

        app_log!(info, 
            "Database connection established: {}",
            database_path.display()
        );

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    /// Get pool reference for custom operations
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
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
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_email ON tenants(email);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_domain ON tenants(domain);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_tenant_name ON tenants(tenant_name);")
            .execute(&self.pool)
            .await?;

        app_log!(info, "Database migrations completed");
        Ok(())
    }

    /// Execute a transaction with automatic rollback on error
    pub async fn transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: for<'c> FnOnce(
            &'c mut sqlx::Transaction<'_, sqlx::Sqlite>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'c>,
        >,
    {
        let mut tx = self.pool.begin().await?;
        match operation(&mut tx).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database health check failed")?;
        Ok(())
    }
}

