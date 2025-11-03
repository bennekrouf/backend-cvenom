// src/tenant_cli.rs
use crate::database::{DatabaseConfig, TenantRepository, TenantService};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use graflog::app_log;

#[derive(Parser)]
#[command(name = "tenant-manager")]
#[command(about = "Manage tenants for the CV generator")]
pub struct TenantCli {
    #[command(subcommand)]
    pub command: TenantCommand,

    #[arg(long, default_value = "data/tenants.db")]
    pub database_path: PathBuf,
}

#[derive(Subcommand)]
pub enum TenantCommand {
    /// Add a new tenant for specific email
    Add { email: String, tenant_name: String },
    /// Add a new tenant for entire domain
    AddDomain { domain: String, tenant_name: String },
    /// Remove/deactivate a tenant by email
    Remove { email: String },
    /// Remove/deactivate a tenant by domain  
    RemoveDomain { domain: String },
    /// List all active tenants
    List,
    /// Check if an email is authorized
    Check { email: String },
    /// Import tenants from a CSV file
    Import { csv_file: PathBuf },
    /// Initialize the database
    Init,
}

pub async fn handle_tenant_command(cli: TenantCli) -> Result<()> {
    // Initialize database
    let mut db_config = DatabaseConfig::new(cli.database_path.clone());
    db_config.init_pool().await?;
    db_config.migrate().await?;

    let pool = db_config.pool()?;
    let tenant_service = TenantService::new(pool);
    let tenant_repo = TenantRepository::new(pool);

    match cli.command {
        TenantCommand::Add { email, tenant_name } => {
            match tenant_repo.create_email_tenant(&email, &tenant_name).await {
                Ok(tenant) => {
                    app_log!(info, "✅ Email tenant created successfully:");
                    app_log!(info, "   Email: {}", email);
                    app_log!(info, "   Tenant: {}", tenant.tenant_name);
                    app_log!(info, "   ID: {}", tenant.id);
                }
                Err(e) => {
                    app_log!(error, "Failed to create tenant: {}", e);
                    if e.to_string().contains("UNIQUE constraint failed") {
                        app_log!(info, "❌ Error: Email '{}' already exists", email);
                    } else {
                        app_log!(info, "❌ Error: {}", e);
                    }
                }
            }
        }

        TenantCommand::AddDomain {
            domain,
            tenant_name,
        } => {
            match tenant_repo
                .create_domain_tenant(&domain, &tenant_name)
                .await
            {
                Ok(tenant) => {
                    app_log!(info, "✅ Domain tenant created successfully:");
                    app_log!(info, "   Domain: @{}", domain);
                    app_log!(info, "   Tenant: {}", tenant.tenant_name);
                    app_log!(info, "   ID: {}", tenant.id);
                    app_log!(info, 
                        "   All emails with @{} can now access tenant '{}'",
                        domain, tenant_name
                    );
                }
                Err(e) => {
                    app_log!(error, "Failed to create domain tenant: {}", e);
                    app_log!(info, "❌ Error: {}", e);
                }
            }
        }

        TenantCommand::Remove { email } => match tenant_repo.deactivate_by_email(&email).await {
            Ok(true) => {
                app_log!(info, "✅ Tenant deactivated for email: {}", email);
            }
            Ok(false) => {
                app_log!(info, "❌ No active tenant found for email: {}", email);
            }
            Err(e) => {
                app_log!(error, "Failed to deactivate tenant: {}", e);
                app_log!(info, "❌ Error: {}", e);
            }
        },

        TenantCommand::RemoveDomain { domain } => {
            match tenant_repo.deactivate_by_domain(&domain).await {
                Ok(true) => {
                    app_log!(info, "✅ Tenant deactivated for domain: @{}", domain);
                }
                Ok(false) => {
                    app_log!(info, "❌ No active tenant found for domain: @{}", domain);
                }
                Err(e) => {
                    app_log!(error, "Failed to deactivate domain tenant: {}", e);
                    app_log!(info, "❌ Error: {}", e);
                }
            }
        }

        TenantCommand::List => match tenant_repo.list_active().await {
            Ok(tenants) => {
                if tenants.is_empty() {
                    app_log!(info, "No active tenants found.");
                } else {
                    app_log!(info, "Active tenants:");
                    app_log!(info, 
                        "{:<5} {:<25} {:<15} {:<20} {:<20}",
                        "ID", "Email/Domain", "Type", "Tenant", "Created"
                    );
                    app_log!(info, "{}", "-".repeat(85));

                    for tenant in tenants {
                        let auth_info = if let Some(email) = &tenant.email {
                            (email.clone(), "Email".to_string())
                        } else if let Some(domain) = &tenant.domain {
                            (format!("@{}", domain), "Domain".to_string())
                        } else {
                            ("Invalid".to_string(), "Error".to_string())
                        };

                        app_log!(info, 
                            "{:<5} {:<25} {:<15} {:<20} {:<20}",
                            tenant.id,
                            auth_info.0,
                            auth_info.1,
                            tenant.tenant_name,
                            tenant.created_at.format("%Y-%m-%d %H:%M")
                        );
                    }
                }
            }
            Err(e) => {
                app_log!(error, "Failed to list tenants: {}", e);
                app_log!(info, "❌ Error: {}", e);
            }
        },

        TenantCommand::Check { email } => match tenant_service.validate_user_access(&email).await {
            Ok(Some(tenant)) => {
                let auth_type = if tenant.email.is_some() {
                    "email"
                } else {
                    "domain"
                };
                app_log!(info, 
                    "✅ Email '{}' is authorized for tenant: {} (via {})",
                    email, tenant.tenant_name, auth_type
                );
                app_log!(info, "   Tenant ID: {}", tenant.id);
                app_log!(info, 
                    "   Created: {}",
                    tenant.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
            Ok(None) => {
                app_log!(info, 
                    "❌ Email '{}' is not authorized (no matching email or domain)",
                    email
                );
            }
            Err(e) => {
                app_log!(error, "Failed to check email: {}", e);
                app_log!(info, "❌ Error: {}", e);
            }
        },

        TenantCommand::Import { csv_file } => {
            if !csv_file.exists() {
                app_log!(info, "❌ CSV file not found: {}", csv_file.display());
                return Ok(());
            }

            let content = tokio::fs::read_to_string(&csv_file).await?;
            let mut reader = csv::Reader::from_reader(content.as_bytes());

            let mut success_count = 0;
            let mut error_count = 0;

            for result in reader.records() {
                match result {
                    Ok(record) => {
                        if record.len() >= 2 {
                            let email = record.get(0).unwrap_or("").trim();
                            let tenant_name = record.get(1).unwrap_or("").trim();

                            if email.is_empty() || tenant_name.is_empty() {
                                error_count += 1;
                                app_log!(info, "⚠️  Skipping empty email or tenant name");
                                continue;
                            }

                            match tenant_repo.create_email_tenant(email, tenant_name).await {
                                Ok(_) => {
                                    success_count += 1;
                                    app_log!(info, "✅ Added: {} -> {}", email, tenant_name);
                                }
                                Err(e) => {
                                    error_count += 1;
                                    if e.to_string().contains("UNIQUE constraint failed") {
                                        app_log!(info, "⚠️  Skipped (already exists): {}", email);
                                    } else {
                                        app_log!(info, "❌ Failed to add {}: {}", email, e);
                                    }
                                }
                            }
                        } else {
                            error_count += 1;
                            app_log!(info, "⚠️  Skipping invalid record (need email,tenant_name)");
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        app_log!(info, "❌ CSV parsing error: {}", e);
                    }
                }
            }

            app_log!(info, "\nImport completed:");
            app_log!(info, "  ✅ Success: {}", success_count);
            app_log!(info, "  ❌ Errors:  {}", error_count);
        }

        TenantCommand::Init => {
            app_log!(info, 
                "✅ Database initialized at: {}",
                cli.database_path.display()
            );
            app_log!(info, "   Tables created: tenants (with email and domain support)");
            app_log!(info, "   Ready to accept tenant registrations");
            app_log!(info, "");
            app_log!(info, "Usage:");
            app_log!(info, "  cargo run -- tenant add <email> <tenant-name>           # Add email-specific tenant");
            app_log!(info, "  cargo run -- tenant add-domain <domain> <tenant-name>   # Add domain tenant (e.g., keyteo.ch)");
            app_log!(info, 
                "  cargo run -- tenant check <email>                       # Check authorization"
            );
        }
    }

    Ok(())
}
