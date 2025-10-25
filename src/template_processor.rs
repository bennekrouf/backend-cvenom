// src/template_processor.rs
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct TemplateProcessor {
    templates_dir: PathBuf,
}

impl TemplateProcessor {
    pub fn new(templates_dir: PathBuf) -> Self {
        Self { templates_dir }
    }

    pub fn process_variables(content: &str, vars: &HashMap<String, String>) -> String {
        vars.iter().fold(content.to_string(), |acc, (key, value)| {
            acc.replace(&format!("{{{{{}}}}}", key), value)
        })
    }

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

    pub fn create_person_from_templates(
        &self,
        person_name: &str,
        data_dir: &PathBuf,
        display_name: Option<&str>,
    ) -> Result<()> {
        let person_dir = data_dir.join(person_name);
        fs::create_dir_all(&person_dir).context("Failed to create person directory")?;

        let mut variables = HashMap::new();
        variables.insert(
            "name".to_string(),
            display_name.unwrap_or(person_name).to_string(),
        );

        // Process and create cv_params.toml
        let toml_template_path = self.templates_dir.join("person_template.toml");
        if toml_template_path.exists() {
            let template_content = fs::read_to_string(&toml_template_path)
                .context("Failed to read person_template.toml")?;
            let processed_content = self.process_template(&template_content, &variables);

            let output_path = person_dir.join("cv_params.toml");
            fs::write(&output_path, processed_content).context("Failed to write cv_params.toml")?;
        }

        // Create experience files for all supported languages
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

        // Create README
        let readme_path = person_dir.join("README.md");
        let readme_content = format!(
            "# {} CV Data\n\nAdd your profile image as `profile.png` in this directory.\nAdd your company logo as `company_logo.png` (optional).\n\nEdit the following files:\n- `cv_params.toml` - Personal information, skills, and key insights\n- `experiences_*.typ` - Work experience for each language (en/fr)\n",
            person_name
        );
        fs::write(&readme_path, readme_content).context("Failed to write README.md")?;

        Ok(())
    }
}
