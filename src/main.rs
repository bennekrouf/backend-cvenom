use anyhow::Result;
use clap::{Parser, Subcommand};
use cv_generator::{list_persons, list_templates, web::start_web_server, CvConfig, CvGenerator};
use std::path::PathBuf;

// mod auth;
mod database;
mod template_system;
mod tenant_cli;

use tenant_cli::{handle_tenant_command, TenantCli};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "data")]
    data_dir: PathBuf,

    #[arg(short, long, default_value = "output")]
    output_dir: PathBuf,

    #[arg(short, long, default_value = "templates")]
    templates_dir: PathBuf,
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            person,
            lang,
            template,
            watch,
            tenant,
        } => {
            let mut config = CvConfig::new(&person, &lang)
                .with_template(template)
                .with_output_dir(cli.output_dir)
                .with_templates_dir(cli.templates_dir);

            // If tenant is specified, use tenant-specific data directory
            if let Some(tenant_name) = tenant {
                let tenant_data_dir = cli.data_dir.join("tenants").join(&tenant_name);
                config = config.with_data_dir(tenant_data_dir.clone());
                println!(
                    "Using tenant-specific data directory: {}",
                    tenant_data_dir.display()
                );
            } else {
                config = config.with_data_dir(cli.data_dir);
            }

            let generator = CvGenerator::new(config)?;

            if watch {
                generator.watch()
            } else {
                generator.generate().map(|_| ())
            }
        }

        Commands::Create { person } => {
            let config = CvConfig::new(&person, "en")
                .with_data_dir(cli.data_dir)
                .with_output_dir(cli.output_dir)
                .with_templates_dir(cli.templates_dir);

            let generator = CvGenerator::new(config)?;
            generator.create_person_unchecked()
        }

        Commands::Generate {
            person,
            lang,
            template,
            watch,
            tenant,
        } => {
            let template_id = template; // Now just a string

            let mut config = CvConfig::new(&person, &lang)
                .with_template(template_id)
                .with_output_dir(cli.output_dir)
                .with_templates_dir(cli.templates_dir);

            // If tenant is specified, use tenant-specific data directory
            if let Some(tenant_name) = tenant {
                let tenant_data_dir = cli.data_dir.join("tenants").join(&tenant_name);
                config = config.with_data_dir(tenant_data_dir.clone());
                println!(
                    "Using tenant-specific data directory: {}",
                    tenant_data_dir.display()
                );
            } else {
                config = config.with_data_dir(cli.data_dir);
            }

            let generator = CvGenerator::new(config)?;

            if watch {
                generator.watch()
            } else {
                generator.generate().map(|_| ())
            }
        }

        Commands::List => {
            let persons = list_persons(&cli.data_dir)?;

            if persons.is_empty() {
                println!("No persons found in {}", cli.data_dir.display());
            } else {
                println!("Available persons:");
                for person in persons {
                    println!("  {}", person);
                }
            }

            Ok(())
        }

        Commands::ListTemplates => {
            let templates = list_templates(&cli.templates_dir)?;

            if templates.is_empty() {
                println!("No templates found in {}", cli.templates_dir.display());
            } else {
                println!("Available templates:");
                for template in templates {
                    println!("  {}", template);
                }
            }

            Ok(())
        }

        Commands::Server { port: _ } => {
            println!("Starting Multi-tenant CV Generator API Server on http://0.0.0.0:4002");
            println!("");
            println!("Multi-tenancy: Users must be registered in SQLite database");
            println!("Authentication: Firebase ID tokens + tenant validation required");
            println!("Database: {}/tenants.db", cli.data_dir.display());
            println!("");
            println!("Public Endpoints:");
            println!("  GET  /api/health        - Health check");
            println!("  GET  /api/templates     - List available templates");
            println!("");
            println!("Protected Endpoints (require Firebase auth + tenant registration):");
            println!("  POST /api/generate      - Generate CV (tenant-isolated)");
            println!("  POST /api/create        - Create person (tenant-isolated)");
            println!("  POST /api/upload-picture - Upload profile picture (tenant-isolated)");
            println!("  GET  /api/me            - Get current user + tenant info");
            println!("");
            println!("Tenant Management:");
            println!("  Use: cargo run -- tenant add <email> <tenant-name>");
            println!("  Use: cargo run -- tenant list");
            println!("");

            start_web_server(cli.data_dir, cli.output_dir, cli.templates_dir).await
        }

        Commands::Tenant { tenant_cli } => handle_tenant_command(tenant_cli).await,
    }
}
