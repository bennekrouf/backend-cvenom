// src/main.rs - Updated to use environment configuration
use anyhow::Result;
use clap::{Parser, Subcommand};
// use cv_generator::font_validator;
use cv_generator::utils::normalize_person_name;
use cv_generator::{list_persons, list_templates, web::start_web_server, CvConfig, CvGenerator};
use std::path::PathBuf;

mod database;
mod environment; // Add this
mod template_system;
mod tenant_cli;

use environment::EnvironmentConfig;
use tenant_cli::{handle_tenant_command, TenantCli};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    // These become optional with environment config
    #[arg(long, help = "Override tenant data directory")]
    data_dir: Option<PathBuf>,

    #[arg(long, help = "Override output directory")]
    output_dir: Option<PathBuf>,

    #[arg(long, help = "Override templates directory")]
    templates_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        person: String,
        #[arg(short, long, default_value = "en")]
        lang: String,
        #[arg(short, long, default_value = "default")]
        template: String,
        #[arg(short, long)]
        watch: bool,
        #[arg(long)]
        tenant: Option<String>,
    },
    Create {
        person: String,
    },
    List,
    ListTemplates,
    Server {
        #[arg(short, long, default_value = "4002")]
        port: u16,
    },
    /// Manage tenants (add, remove, list, etc.)
    Tenant {
        #[command(flatten)]
        tenant_cli: TenantCli,
    },
    /// Show current configuration
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment configuration
    let env_config = EnvironmentConfig::load()?;
    env_config.ensure_directories().await?;

    let cli = Cli::parse();

    // Use CLI overrides if provided, otherwise use environment config
    let data_dir = cli.data_dir.unwrap_or(env_config.tenant_data_path.clone());
    let output_dir = cli.output_dir.unwrap_or(env_config.output_path.clone());
    let templates_dir = cli
        .templates_dir
        .unwrap_or(env_config.templates_path.clone());

    match cli.command {
        Commands::Config => {
            println!("Current configuration:");
            println!(
                "  Environment: {}",
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string())
            );
            println!(
                "  Tenant Data Path: {}",
                env_config.tenant_data_path.display()
            );
            println!("  Output Path: {}", env_config.output_path.display());
            println!("  Templates Path: {}", env_config.templates_path.display());
            println!("  Database Path: {}", env_config.database_path.display());
            println!();
            println!("Override with CLI:");
            println!("  --data-dir <path>");
            println!("  --output-dir <path>");
            println!("  --templates-dir <path>");
            println!();
            println!("Override with environment variables:");
            println!("  ENVIRONMENT=production|local");
            println!("  CVENOM_TENANT_DATA_PATH=/path/to/tenant/data");
            println!("  CVENOM_OUTPUT_PATH=/path/to/output");
            println!("  CVENOM_TEMPLATES_PATH=/path/to/templates");
            println!("  CVENOM_DATABASE_PATH=/path/to/database.db");
            Ok(())
        }

        Commands::Generate {
            person,
            lang,
            template,
            watch,
            tenant,
        } => {
            let mut config = CvConfig::new(&person, &lang)
                .with_template(template)
                .with_output_dir(output_dir)
                .with_templates_dir(templates_dir);

            // If tenant is specified, use tenant-specific data directory
            if let Some(tenant_name) = tenant {
                let tenant_data_dir = data_dir.join(&tenant_name);
                config = config.with_data_dir(tenant_data_dir.clone());
                println!(
                    "Using tenant-specific data directory: {}",
                    tenant_data_dir.display()
                );
            } else {
                config = config.with_data_dir(data_dir);
            }

            let generator = CvGenerator::new(config)?;

            if watch {
                generator.watch()
            } else {
                generator.generate().map(|_| ())
            }
        }

        Commands::Create { person } => {
            let normalized_person = normalize_person_name(&person);

            let config = CvConfig::new(&normalized_person, "en")
                .with_data_dir(data_dir)
                .with_output_dir(output_dir)
                .with_templates_dir(templates_dir);

            let generator = CvGenerator::new(config)?;
            generator.create_person_unchecked()
        }

        Commands::List => {
            let persons = list_persons(&data_dir)?;

            if persons.is_empty() {
                println!("No persons found in {}", data_dir.display());
            } else {
                println!("Available persons:");
                for person in persons {
                    println!("  {}", person);
                }
            }

            Ok(())
        }

        Commands::ListTemplates => {
            let templates = list_templates(&templates_dir)?;

            if templates.is_empty() {
                println!("No templates found in {}", templates_dir.display());
            } else {
                println!("Available templates:");
                for template in templates {
                    println!("  {}", template);
                }
            }

            Ok(())
        }

        Commands::Server { port: _ } => {
            println!("Starting Multi-tenant CV Generator API Server");
            println!(
                "Environment: {}",
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string())
            );
            println!("Tenant Data: {}", env_config.tenant_data_path.display());
            println!("Database: {}", env_config.database_path.display());
            println!();

            // Validate fonts before starting server
            // let font_config_path = Some(PathBuf::from("font_validation.yaml"));
            // crate::font_validator::validate_fonts_or_exit(font_config_path).await?;

            println!("Server: http://0.0.0.0:4002");
            println!();

            start_web_server(
                env_config.tenant_data_path,
                env_config.output_path,
                env_config.templates_path,
            )
            .await
        }

        Commands::Tenant { mut tenant_cli } => {
            // Override database path with environment config
            tenant_cli.database_path = env_config.database_path;
            handle_tenant_command(tenant_cli).await
        }
    }
}
