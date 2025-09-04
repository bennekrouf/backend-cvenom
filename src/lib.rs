// Updated CvConfig to support templates
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;
use std::{fs, path::PathBuf};

pub mod auth;
pub mod database;
pub mod web;

/// Available CV templates
#[derive(Debug, Clone)]
pub enum CvTemplate {
    Default,
    Keyteo,
    KeyteoFull,
}

impl CvTemplate {
    pub fn as_str(&self) -> &str {
        match self {
            CvTemplate::Default => "default",
            CvTemplate::Keyteo => "keyteo",
            CvTemplate::KeyteoFull => "keyteo_full",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "default" => Ok(CvTemplate::Default),
            "keyteo" => Ok(CvTemplate::Keyteo),
            "keyteo_full" => Ok(CvTemplate::KeyteoFull),
            _ => anyhow::bail!("Unsupported template: {}. Use default, keyteo", s),
        }
    }

    pub fn template_file(&self) -> &str {
        match self {
            CvTemplate::Default => "cv.typ",
            CvTemplate::Keyteo => "cv_keyteo.typ",
            CvTemplate::KeyteoFull => "cv_keyteo_full.typ",
        }
    }

    pub fn all() -> Vec<&'static str> {
        vec!["default", "keyteo", "keyteo_full"]
    }
}

/// Template processing for creating new persons
pub struct TemplateProcessor {
    templates_dir: PathBuf,
}

impl TemplateProcessor {
    pub fn new(templates_dir: PathBuf) -> Self {
        Self { templates_dir }
    }

    /// Process a template file by replacing placeholders
    pub fn process_template(
        &self,
        template_content: &str,
        variables: &HashMap<String, String>,
    ) -> String {
        let mut result = template_content.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Create person directory with template-based files
    // Lines to adapt in src/lib.rs - Around line 82, update create_person_from_templates:
    pub fn create_person_from_templates(
        &self,
        person_name: &str,
        data_dir: &PathBuf,
    ) -> Result<()> {
        let person_dir = data_dir.join(person_name);
        fs::create_dir_all(&person_dir).context("Failed to create person directory")?;

        let mut variables = HashMap::new();
        variables.insert("name".to_string(), person_name.to_string());

        // Process and create cv_params.toml
        let toml_template_path = self.templates_dir.join("person_template.toml");
        if toml_template_path.exists() {
            let template_content = fs::read_to_string(&toml_template_path)
                .context("Failed to read person_template.toml")?;
            let processed_content = self.process_template(&template_content, &variables);

            let output_path = person_dir.join("cv_params.toml");
            fs::write(&output_path, processed_content).context("Failed to write cv_params.toml")?;
        }

        // Create experience files for all supported languages with new structured format
        let experience_template_path = self.templates_dir.join("experiences_template.typ");
        if experience_template_path.exists() {
            let template_content = fs::read_to_string(&experience_template_path)
                .context("Failed to read experiences_template.typ")?;

            let languages = ["en", "fr"];
            for lang in &languages {
                let output_path = person_dir.join(format!("experiences_{}.typ", lang));
                fs::write(&output_path, &template_content)
                    .with_context(|| format!("Failed to write experiences_{}.typ", lang))?;
            }
        }

        // Create placeholder profile image info
        let readme_path = person_dir.join("README.md");
        let readme_content = format!(
            "# {} CV Data\n\nAdd your profile image as `profile.png` in this directory.\nAdd your company logo as `company_logo.png` (optional - uses tenant-wide logo if not provided).\n\nEdit the following files:\n- `cv_params.toml` - Personal information, skills, and key insights\n- `experiences_*.typ` - Work experience for each language (en/fr)\n\n## Experience Structure\nEach role now uses structured_experience() with:\n- **Context**: Background info (company size, tech stack, business domain)\n- **Responsibilities**: Specific achievements and duties with metrics\n\nLanguage selection is done at generation time via API.\n\n## Available Templates:\n- default: Standard CV layout\n- keyteo: CV with Keyteo logo and professional branding\n\n## Tips:\n- Use bullet points for context and responsibilities arrays\n- Include specific metrics and technologies where possible\n- Keep context brief (1-3 points), responsibilities more detailed (3-5 points)\n",
            person_name
        );
        fs::write(&readme_path, readme_content).context("Failed to write README.md")?;

        Ok(())
    }
}

/// Multi-tenant CV configuration
pub struct CvConfig {
    pub person_name: String,
    pub lang: String,
    pub template: CvTemplate,
    pub output_dir: PathBuf,
    pub data_dir: PathBuf,
    pub templates_dir: PathBuf,
}

impl CvConfig {
    pub fn new(person_name: &str, lang: &str) -> Self {
        Self {
            person_name: person_name.to_string(),
            lang: lang.to_string(),
            template: CvTemplate::Default,
            output_dir: PathBuf::from("output"),
            data_dir: PathBuf::from("data"),
            templates_dir: PathBuf::from("templates"),
        }
    }

    pub fn with_template(mut self, template: CvTemplate) -> Self {
        self.template = template;
        self
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
        self.person_data_dir()
            .join(format!("experiences_{}.typ", self.lang))
    }

    /// Get person's profile image path
    pub fn person_image_path(&self) -> PathBuf {
        self.person_data_dir().join("profile.png")
    }

    /// Get the template file to use for compilation
    pub fn template_file_path(&self) -> PathBuf {
        self.templates_dir.join(self.template.template_file())
    }
}

/// Multi-tenant CV Generator
pub struct CvGenerator {
    pub config: CvConfig,
}

impl CvGenerator {
    pub fn new(config: CvConfig) -> Result<Self> {
        let person_dir = config.person_data_dir();
        println!(
            "DEBUG: Looking for person directory at: {}",
            person_dir.display()
        );

        // Validate language
        if !["fr", "en"].contains(&config.lang.as_str()) {
            anyhow::bail!("Unsupported language: {}. Use fr, en", config.lang);
        }

        // Check if person's data directory exists
        let person_dir = config.person_data_dir();
        if !person_dir.exists() {
            anyhow::bail!(
                "Person directory not found: {}. Create it with required files.",
                person_dir.display()
            );
        }

        // Validate required files exist
        let config_path = config.person_config_path();
        let experiences_path = config.person_experiences_path();
        let template_path = config.template_file_path();

        if !config_path.exists() {
            anyhow::bail!("Config file not found: {}", config_path.display());
        }

        if !experiences_path.exists() {
            anyhow::bail!("Experiences file not found: {}", experiences_path.display());
        }

        if !template_path.exists() {
            anyhow::bail!("Template file not found: {}", template_path.display());
        }

        Ok(Self { config })
    }

    /// Generate the CV PDF
    pub fn generate(&self) -> Result<PathBuf> {
        self.setup_output_dir()?;
        self.prepare_workspace()?;

        let output_path = self.compile_cv()?;

        self.cleanup_workspace()?;

        println!(
            "✅ Successfully compiled CV for {} ({} template, {} lang) to {}",
            self.config.person_name,
            self.config.template.as_str(),
            self.config.lang,
            output_path.display()
        );

        Ok(output_path)
    }

    /// Watch for changes and regenerate
    pub fn watch(&self) -> Result<()> {
        self.setup_output_dir()?;
        self.prepare_workspace()?;

        let output_path = self.config.output_dir.join(format!(
            "{}_{}_{}.pdf",
            self.config.person_name,
            self.config.template.as_str(),
            self.config.lang
        ));

        println!(
            "👀 Watching for changes for {} ({} template)...",
            self.config.person_name,
            self.config.template.as_str()
        );

        let status = Command::new("typst")
            .arg("watch")
            .arg(self.config.template.template_file())
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
        template_processor
            .create_person_from_templates(&self.config.person_name, &self.config.data_dir)?;

        let person_dir = self.config.person_data_dir();
        println!(
            "Created person directory structure for: {}",
            self.config.person_name
        );
        println!("  Directory: {}", person_dir.display());
        println!("  Files created:");
        println!("    - cv_params.toml (edit your personal info)");
        println!("    - experiences_*.typ (for all languages)");
        println!("    - README.md (instructions)");
        println!("  Available templates: {}", CvTemplate::all().join(", "));
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
        println!("Setting up directories...");
        fs::create_dir_all(&self.config.output_dir).context("Failed to create output directory")?;

        // Create temporary workspace directory
        fs::create_dir_all("tmp_workspace")
            .context("Failed to create temporary workspace directory")?;

        Ok(())
    }

    // In src/lib.rs, modify prepare_workspace method to look for logo in tenant directory
    fn prepare_workspace(&self) -> Result<()> {
        println!("Preparing workspace in tmp_workspace/...");

        // Change to temporary workspace directory
        std::env::set_current_dir("tmp_workspace")
            .context("Failed to change to temporary workspace")?;

        // Copy person's config to workspace
        let config_source = PathBuf::from("..").join(self.config.person_config_path());
        let config_dest = PathBuf::from("cv_params.toml");
        println!(
            "Copying config from {} to {}",
            config_source.display(),
            config_dest.display()
        );
        fs::copy(&config_source, &config_dest).context("Failed to copy person config")?;

        // Copy person's experiences file for the requested language
        let exp_source = PathBuf::from("..").join(self.config.person_experiences_path());
        let exp_dest = PathBuf::from("experiences.typ"); // Single standardized name
        println!(
            "Copying experiences from {} to {}",
            exp_source.display(),
            exp_dest.display()
        );
        fs::copy(&exp_source, &exp_dest).context("Failed to copy person experiences")?;

        // Copy person's profile image
        let person_image_png = PathBuf::from("..").join(self.config.person_image_path());
        if person_image_png.exists() {
            let profile_dest = PathBuf::from("profile.png");
            println!(
                "Copying profile image from {} to {}",
                person_image_png.display(),
                profile_dest.display()
            );

            match fs::copy(&person_image_png, &profile_dest) {
                Ok(_) => {
                    // Verify the copied image is valid
                    match std::process::Command::new("file")
                        .arg(&profile_dest)
                        .output()
                    {
                        Ok(output) => {
                            let file_type = String::from_utf8_lossy(&output.stdout);
                            println!("Profile image file type: {}", file_type.trim());
                            if !file_type.contains("PNG") && !file_type.contains("JPEG") {
                                println!("Warning: Profile image may not be a valid image format");
                            }
                        }
                        Err(e) => println!("Could not verify image type: {}", e),
                    }
                }
                Err(e) => {
                    println!("Failed to copy profile image: {}", e);
                }
            }
        } else {
            println!("No profile image found at {}", person_image_png.display());
        }

        // Look for company logo in tenant directory first (shared), then person directory (override)
        let tenant_logo_source = PathBuf::from("..")
            .join(&self.config.data_dir)
            .join("company_logo.png");
        let person_logo_source = PathBuf::from("..")
            .join(self.config.person_data_dir())
            .join("company_logo.png");
        let logo_dest = PathBuf::from("company_logo.png");

        println!(
            "DEBUG: Looking for logo at tenant dir: {}",
            tenant_logo_source.display()
        );
        println!(
            "DEBUG: Looking for logo at person dir: {}",
            person_logo_source.display()
        );

        let _logo_available = if person_logo_source.exists() {
            println!(
                "Copying person logo from {} to {}",
                person_logo_source.display(),
                logo_dest.display()
            );
            match fs::copy(&person_logo_source, &logo_dest) {
                Ok(_) => {
                    println!("Person logo copied successfully");
                    true
                }
                Err(e) => {
                    println!("Failed to copy person logo: {}", e);
                    false
                }
            }
        } else if tenant_logo_source.exists() {
            println!(
                "No person logo found, using tenant logo from {} to {}",
                tenant_logo_source.display(),
                logo_dest.display()
            );
            match fs::copy(&tenant_logo_source, &logo_dest) {
                Ok(_) => {
                    println!("Tenant logo copied successfully");
                    true
                }
                Err(e) => {
                    println!("Failed to copy tenant logo: {}", e);
                    false
                }
            }
        } else {
            println!(
                "No logo found at either {} or {} - will use fallback",
                tenant_logo_source.display(),
                person_logo_source.display()
            );
            false
        };

        // Use tenant-specific template if available, fallback to default template
        let tenant = &self
            .config
            .data_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("default");

        let tenant_template = PathBuf::from("..")
            .join(&self.config.templates_dir)
            .join(format!("cv_{}.typ", tenant));
        let template_file = if tenant_template.exists() {
            println!(
                "Using tenant-specific template: {}",
                tenant_template.display()
            );
            tenant_template
        } else {
            println!("Using default template");
            PathBuf::from("..").join(&self.config.template_file_path())
        };

        let template_dest = PathBuf::from(self.config.template.template_file());

        println!(
            "Copying template from {} to {}",
            template_file.display(),
            template_dest.display()
        );

        if template_file.exists() {
            let template_content =
                fs::read_to_string(&template_file).context("Failed to read template file")?;

            fs::write(&template_dest, template_content)
                .context("Failed to write processed template file")?;

            println!(
                "Template file written to workspace: {}",
                template_dest.display()
            );
        } else {
            return Err(anyhow::anyhow!(
                "Template file not found: {}",
                template_file.display()
            ));
        }

        // Copy base template.typ if it exists
        let base_template = PathBuf::from("..")
            .join(&self.config.templates_dir)
            .join("template.typ");
        let base_dest = PathBuf::from("template.typ");
        if base_template.exists() {
            println!(
                "Copying base template from {} to {}",
                base_template.display(),
                base_dest.display()
            );
            fs::copy(&base_template, &base_dest).context("Failed to copy template.typ")?;
        }

        // Debug: Show workspace contents
        println!("DEBUG: Current workspace contents:");
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        }

        Ok(())
    }

    /// Generate CV and return PDF data directly (for web API)
    pub fn generate_pdf_data(&self) -> Result<Vec<u8>> {
        self.setup_output_dir()?;
        self.prepare_workspace()?;

        let output_path = self.compile_cv()?;

        // Read PDF data directly
        let pdf_data = fs::read(&output_path).context("Failed to read generated PDF")?;

        self.cleanup_workspace()?;

        println!(
            "Successfully generated PDF for {} ({} template, {} lang)",
            self.config.person_name,
            self.config.template.as_str(),
            self.config.lang
        );

        Ok(pdf_data)
    }

    fn compile_cv(&self) -> Result<PathBuf> {
        let output_path = PathBuf::from("..")
            .join(&self.config.output_dir)
            .join(format!(
                "{}_{}_{}.pdf",
                self.config.person_name,
                self.config.template.as_str(),
                self.config.lang
            ));

        let mut cmd = Command::new("typst");
        cmd.arg("compile")
            .arg(self.config.template.template_file())
            .arg(&output_path);

        // Pass language as input to Typst
        cmd.arg("--input").arg(format!("lang={}", self.config.lang));

        // Add input files so Typst can access them via sys.inputs
        if PathBuf::from("company_logo.png").exists() {
            cmd.arg("--input").arg("company_logo.png=company_logo.png");
        }

        if PathBuf::from("profile.png").exists() {
            cmd.arg("--input").arg("profile.png=profile.png");
        }

        let output = cmd.output().context("Failed to execute typst command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            eprintln!("Typst compilation failed:");
            eprintln!("STDERR: {}", stderr);
            eprintln!("STDOUT: {}", stdout);
            eprintln!("Command: {:?}", cmd);

            anyhow::bail!(
                "Typst compilation failed: stderr={}, stdout={}",
                stderr,
                stdout
            );
        }

        Ok(output_path)
    }

    fn cleanup_workspace(&self) -> Result<()> {
        // Change back to root directory
        std::env::set_current_dir("..").context("Failed to change back to root directory")?;

        println!("Cleaning up temporary workspace...");

        // Remove entire temporary workspace directory
        if PathBuf::from("tmp_workspace").exists() {
            fs::remove_dir_all("tmp_workspace")
                .context("Failed to remove temporary workspace directory")?;
            println!("Temporary workspace cleaned up");
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

    let entries = fs::read_dir(data_dir).context("Failed to read data directory")?;

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

/// List all available templates
pub fn list_templates(templates_dir: &PathBuf) -> Result<Vec<String>> {
    let mut templates = Vec::new();

    for template in CvTemplate::all() {
        let template_path = templates_dir.join(match template {
            "default" => "cv.typ",
            "keyteo" => "cv_keyteo.typ",
            "keyteo_full" => "cv_keyteo_full.typ",
            _ => continue,
        });

        if template_path.exists() {
            templates.push(template.to_string());
        }
    }

    if templates.is_empty() {
        templates.push("default".to_string()); // Always have default as fallback
    }

    Ok(templates)
}

/// Convenience function for quick CV generation
pub fn generate_cv(
    person_name: &str,
    lang: &str,
    template: Option<&str>,
    output_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let mut config = CvConfig::new(person_name, lang);

    if let Some(template_str) = template {
        let template = CvTemplate::from_str(template_str)?;
        config = config.with_template(template);
    }

    if let Some(dir) = output_dir {
        config = config.with_output_dir(dir);
    }

    let generator = CvGenerator::new(config)?;
    generator.generate()
}
