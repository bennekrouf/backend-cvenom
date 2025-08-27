use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use cv_generator::{CvConfig, CvGenerator, list_persons};

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
        #[arg(short, long)]
        watch: bool,
    },
    Create {
        person: String,
    },
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Generate { person, lang, watch } => {
            let config = CvConfig::new(&person, &lang)
                .with_data_dir(cli.data_dir)
                .with_output_dir(cli.output_dir)
                .with_templates_dir(cli.templates_dir);
            
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
                .with_templates_dir(cli.templates_dir);
            
            let generator = CvGenerator { config };
            generator.create_person()
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
    }
}
