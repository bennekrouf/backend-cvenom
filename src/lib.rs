use anyhow::{Context, Result};
use std::{fs, path::PathBuf};
use std::process::Command;
use std::collections::HashMap;

pub mod web;

/// Template processing for creating new persons
pub struct TemplateProcessor {
    templates_dir: PathBuf,
}

impl TemplateProcessor {
    pub fn new(templates_dir: PathBuf) -> Self {
        Self { templates_dir }
    }
    
    /// Process a template file by replacing placeholders
    pub fn process_template(&self, template_content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template_content.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
    
    /// Create person directory with template-based files
    pub fn create_person_from_templates(&self, person_name: &str, data_dir: &PathBuf) -> Result<()> {
        let person_dir = data_dir.join(person_name);
        fs::create_dir_all(&person_dir)
            .context("Failed to create person directory")?;
        
        // Create variables for template processing
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), person_name.to_string());
        
        // Process and create cv_params.toml
        let toml_template_path = self.templates_dir.join("person_template.toml");
        if toml_template_path.exists() {
            let template_content = fs::read_to_string(&toml_template_path)
                .context("Failed to read person_template.toml")?;
            let processed_content = self.process_template(&template_content, &variables);
            
            let output_path = person_dir.join("cv_params.toml");
            fs::write(&output_path, processed_content)
                .context("Failed to write cv_params.toml")?;
        }
        
        // Create experience files for all supported languages
        let experience_template_path = self.templates_dir.join("experiences_template.typ");
        if experience_template_path.exists() {
            let template_content = fs::read_to_string(&experience_template_path)
                .context("Failed to read experiences_template.typ")?;
            
            let languages = ["en", "fr", "ch", "ar"];
            for lang in &languages {
                let output_path = person_dir.join(format!("experiences_{}.typ", lang));
                fs::write(&output_path, &template_content)
                    .with_context(|| format!("Failed to write experiences_{}.typ", lang))?;
            }
        }
        
        // Create placeholder profile image info
        let readme_path = person_dir.join("README.md");
        let readme_content = format!(
            "# {} CV Data\n\nAdd your profile image as `profile.png` in this directory.\n\nEdit the following files:\n- `cv_params.toml` - Personal information and skills\n- `experiences_*.typ` - Work experience for each language\n",
            person_name
        );
        fs::write(&readme_path, readme_content)
            .context("Failed to write README.md")?;
        
        Ok(())
    }
}

/// Multi-tenant CV configuration
pub struct CvConfig {
    pub person_name: String,
    pub lang: String,
    pub output_dir: PathBuf,
    pub data_dir: PathBuf,
    pub templates_dir: PathBuf,
}

impl CvConfig {
    pub fn new(person_name: &str, lang: &str) -> Self {
        Self {
            person_name: person_name.to_string(),
            lang: lang.to_string(),
            output_dir: PathBuf::from("output"),
            data_dir: PathBuf::from("data"),
            templates_dir: PathBuf::from("templates"),
        }
    }
    
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }
    
    pub fn with_data_dir(mut self, dir: PathBuf) -> Self {
        self.data_dir = dir;
        self
    }
    
    pub fn with_templates_dir(mut self, dir: PathBuf) -> Self {
        self.templates_dir = dir;
        self
    }
    
    /// Get person's data directory
    pub fn person_data_dir(&self) -> PathBuf {
        self.data_dir.join(&self.person_name)
    }
    
    /// Get person's config file path
    pub fn person_config_path(&self) -> PathBuf {
        self.person_data_dir().join("cv_params.toml")
    }
    
    /// Get person's experiences file path
    pub fn person_experiences_path(&self) -> PathBuf {
        self.person_data_dir().join(format!("experiences_{}.typ", self.lang))
    }

    /// Get person's profile image path
    pub fn person_image_path(&self) -> PathBuf {
        self.person_data_dir().join("profile.jpg")
    }
}

/// Multi-tenant CV Generator
pub struct CvGenerator {
    pub config: CvConfig,
}

impl CvGenerator {
    pub fn new(config: CvConfig) -> Result<Self> {
        // Validate language
        if !["fr", "en", "ch", "ar"].contains(&config.lang.as_str()) {
            anyhow::bail!("Unsupported language: {}. Use fr, en, ch, or ar", config.lang);
        }
        
        // Check if person's data directory exists
        let person_dir = config.person_data_dir();
        if !person_dir.exists() {
            anyhow::bail!("Person directory not found: {}. Create it with required files.", person_dir.display());
        }
        
        // Validate required files exist
        let config_path = config.person_config_path();
        let experiences_path = config.person_experiences_path();
        
        if !config_path.exists() {
            anyhow::bail!("Config file not found: {}", config_path.display());
        }
        
        if !experiences_path.exists() {
            anyhow::bail!("Experiences file not found: {}", experiences_path.display());
        }
        
        Ok(Self { config })
    }
    
    /// Generate the CV PDF
    pub fn generate(&self) -> Result<PathBuf> {
        self.setup_output_dir()?;
        self.prepare_workspace()?;
        
        let output_path = self.compile_cv()?;
        
        self.cleanup_workspace()?;
        
        println!("✓ Successfully compiled CV for {} in {} to {}", 
                self.config.person_name, self.config.lang, output_path.display());
        
        Ok(output_path)
    }
    
    /// Watch for changes and regenerate
    pub fn watch(&self) -> Result<()> {
        self.setup_output_dir()?;
        self.prepare_workspace()?;
        
        let output_path = self.config.output_dir.join(format!("{}_{}.pdf", self.config.person_name, self.config.lang));
        
        println!("👀 Watching for changes for {}...", self.config.person_name);
        
        let status = Command::new("typst")
            .arg("watch")
            .arg("cv.typ")
            .arg(&output_path)
            .status()
            .context("Failed to execute typst watch command")?;

        if !status.success() {
            anyhow::bail!("Typst watch failed");
        }
        
        Ok(())
    }
    
    /// Create person's data directory structure using templates (bypassing validation)
    pub fn create_person_unchecked(&self) -> Result<()> {
        let template_processor = TemplateProcessor::new(self.config.templates_dir.clone());
        template_processor.create_person_from_templates(&self.config.person_name, &self.config.data_dir)?;
        
        let person_dir = self.config.person_data_dir();
        println!("Created person directory structure for: {}", self.config.person_name);
        println!("  Directory: {}", person_dir.display());
        println!("  Files created:");
        println!("    - cv_params.toml (edit your personal info)");
        println!("    - experiences_*.typ (for all languages)");
        println!("    - README.md (instructions)");
        println!("  Next steps:");
        println!("    1. Add your profile image as: profile.png");
        println!("    2. Edit cv_params.toml with your information");
        println!("    3. Update experiences_*.typ files with your work history");
        
        Ok(())
    }
    
    /// Create person's data directory structure using templates
    pub fn create_person(&self) -> Result<()> {
        self.create_person_unchecked()
    }
    
    fn setup_output_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.config.output_dir)
            .context("Failed to create output directory")
    }
    
    fn prepare_workspace(&self) -> Result<()> {
        // Copy person's config to workspace
        fs::copy(self.config.person_config_path(), "cv_params.toml")
        .context("Failed to copy person config")?;
        
    // Copy person's experiences file  
    fs::copy(self.config.person_experiences_path(), format!("experiences_{}.typ", self.config.lang))
        .context("Failed to copy person experiences")?;
        
    // Copy person's profile image - use the actual filename from TOML
    let person_image_png = self.config.person_data_dir().join("profile.png");
    if person_image_png.exists() {
        fs::copy(&person_image_png, "profile.png")
            .context("Failed to copy person image")?;
    }
            
        // Copy template files if they don't exist in workspace
        let template_file = self.config.templates_dir.join("template.typ");
        if template_file.exists() && !PathBuf::from("template.typ").exists() {
            fs::copy(template_file, "template.typ")
                .context("Failed to copy template.typ")?;
        }
        
        let cv_template = self.config.templates_dir.join("cv.typ");
        if cv_template.exists() && !PathBuf::from("cv.typ").exists() {
            fs::copy(cv_template, "cv.typ")
                .context("Failed to copy cv.typ")?;
        }
        
        Ok(())
    }
    
    fn compile_cv(&self) -> Result<PathBuf> {
        let output_path = self.config.output_dir.join(format!("{}_{}.pdf", self.config.person_name, self.config.lang));
        
        let status = Command::new("typst")
            .arg("compile")
            .arg("cv.typ")
            .arg(&output_path)
            .status()
            .context("Failed to execute typst command")?;

        if !status.success() {
            anyhow::bail!("Typst compilation failed");
        }
        
        Ok(output_path)
    }
    
    fn cleanup_workspace(&self) -> Result<()> {
        // Clean up copied files
        let files_to_clean = [
            "cv_params.toml",
            &format!("experiences_{}.typ", self.config.lang),
            "profile.jpg",
        ];
        
        for file in &files_to_clean {
            let path = PathBuf::from(file);
            if path.exists() {
                fs::remove_file(path)
                    .with_context(|| format!("Failed to clean up {}", file))?;
            }
        }
        
        Ok(())
    }
}

/// List all available persons
pub fn list_persons(data_dir: &PathBuf) -> Result<Vec<String>> {
    let mut persons = Vec::new();
    
    if !data_dir.exists() {
        return Ok(persons);
    }
    
    let entries = fs::read_dir(data_dir)
        .context("Failed to read data directory")?;
        
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Check if person has required files
                let config_path = path.join("cv_params.toml");
                if config_path.exists() {
                    persons.push(name.to_string());
                }
            }
        }
    }
    
    persons.sort();
    Ok(persons)
}

/// Convenience function for quick CV generation
pub fn generate_cv(person_name: &str, lang: &str, output_dir: Option<PathBuf>) -> Result<PathBuf> {
    let mut config = CvConfig::new(person_name, lang);
    
    if let Some(dir) = output_dir {
        config = config.with_output_dir(dir);
    }
    
    let generator = CvGenerator::new(config)?;
    generator.generate()
}
