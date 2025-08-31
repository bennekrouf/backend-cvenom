// src/tenant_cli.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::database::{DatabaseConfig, TenantService, TenantRepository};
use tracing::error;

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
    /// Add a new tenant
    Add {
        email: String,
        tenant_name: String,
    },
    /// Remove/deactivate a tenant
    Remove {
        email: String,
    },
    /// List all active tenants
    List,
    /// Check if an email is authorized
    Check {
        email: String,
    },
    /// Import tenants from a CSV file
    Import {
        csv_file: PathBuf,
    },
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
            match tenant_repo.create(&email, &tenant_name).await {
                Ok(tenant) => {
                    println!("✅ Tenant created successfully:");
                    println!("   Email: {}", tenant.email);
                    println!("   Tenant: {}", tenant.tenant_name);
                    println!("   ID: {}", tenant.id);
                },
                Err(e) => {
                    error!("Failed to create tenant: {}", e);
                    if e.to_string().contains("UNIQUE constraint failed") {
                        println!("❌ Error: Email '{}' already exists", email);
                    } else {
                        println!("❌ Error: {}", e);
                    }
                }
            }
        },
        
        TenantCommand::Remove { email } => {
            match tenant_repo.deactivate(&email).await {
                Ok(true) => {
                    println!("✅ Tenant deactivated for email: {}", email);
                },
                Ok(false) => {
                    println!("❌ No active tenant found for email: {}", email);
                },
                Err(e) => {
                    error!("Failed to deactivate tenant: {}", e);
                    println!("❌ Error: {}", e);
                }
            }
        },
        
        TenantCommand::List => {
            match tenant_repo.list_active().await {
                Ok(tenants) => {
                    if tenants.is_empty() {
                        println!("No active tenants found.");
                    } else {
                        println!("Active tenants:");
                        println!("{:<5} {:<30} {:<20} {:<20}", "ID", "Email", "Tenant", "Created");
                        println!("{}", "-".repeat(75));
                        
                        for tenant in tenants {
                            println!("{:<5} {:<30} {:<20} {:<20}", 
                                tenant.id, 
                                tenant.email, 
                                tenant.tenant_name,
                                tenant.created_at.format("%Y-%m-%d %H:%M")
                            );
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to list tenants: {}", e);
                    println!("❌ Error: {}", e);
                }
            }
        },
        
        TenantCommand::Check { email } => {
            match tenant_service.validate_user_access(&email).await {
                Ok(Some(tenant)) => {
                    println!("✅ Email '{}' is authorized for tenant: {}", email, tenant.tenant_name);
                    println!("   Tenant ID: {}", tenant.id);
                    println!("   Created: {}", tenant.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                },
                Ok(None) => {
                    println!("❌ Email '{}' is not authorized (not found or inactive)", email);
                },
                Err(e) => {
                    error!("Failed to check email: {}", e);
                    println!("❌ Error: {}", e);
                }
            }
        },
        
        TenantCommand::Import { csv_file } => {
            if !csv_file.exists() {
                println!("❌ CSV file not found: {}", csv_file.display());
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
                                println!("⚠️  Skipping empty email or tenant name");
                                continue;
                            }
                            
                            match tenant_repo.create(email, tenant_name).await {
                                Ok(_) => {
                                    success_count += 1;
                                    println!("✅ Added: {} -> {}", email, tenant_name);
                                },
                                Err(e) => {
                                    error_count += 1;
                                    if e.to_string().contains("UNIQUE constraint failed") {
                                        println!("⚠️  Skipped (already exists): {}", email);
                                    } else {
                                        println!("❌ Failed to add {}: {}", email, e);
                                    }
                                }
                            }
                        } else {
                            error_count += 1;
                            println!("⚠️  Skipping invalid record (need email,tenant_name)");
                        }
                    },
                    Err(e) => {
                        error_count += 1;
                        println!("❌ CSV parsing error: {}", e);
                    }
                }
            }
            
            println!("\nImport completed:");
            println!("  ✅ Success: {}", success_count);
            println!("  ❌ Errors:  {}", error_count);
        },
        
        TenantCommand::Init => {
            println!("✅ Database initialized at: {}", cli.database_path.display());
            println!("   Tables created: tenants");
            println!("   Ready to accept tenant registrations");
        }
    }
    
    Ok(())
}
