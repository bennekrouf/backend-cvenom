// src/config.rs
// use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct CvConfig {
    pub person_name: String,
    pub lang: String,
    pub template: String,
    pub output_dir: PathBuf,
    pub data_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub root_dir: PathBuf,
}

impl CvConfig {
    pub fn new(person_name: &str, lang: &str) -> Self {
        let normalized_lang = match lang.to_lowercase().as_str() {
            "fr" | "french" | "français" => "fr",
            "en" | "english" | "anglais" => "en",
            _ => "en",
        };

        // Capture the current directory at creation time
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self {
            person_name: person_name.to_string(),
            lang: normalized_lang.to_string(),
            template: "default".to_string(),
            output_dir: PathBuf::from("output"),
            data_dir: PathBuf::from("data"),
            templates_dir: PathBuf::from("templates"),
            root_dir: current_dir,
        }
    }

    pub fn with_template(mut self, template: String) -> Self {
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

    fn absolute_path(&self, relative_path: &PathBuf) -> PathBuf {
        if relative_path.is_absolute() {
            relative_path.clone()
        } else {
            self.root_dir.join(relative_path)
        }
    }

    pub fn data_dir_absolute(&self) -> PathBuf {
        self.absolute_path(&self.data_dir)
    }

    pub fn person_data_dir(&self) -> PathBuf {
        self.absolute_path(&self.data_dir.join(&self.person_name))
    }

    pub fn person_config_path(&self) -> PathBuf {
        self.person_data_dir().join("cv_params.toml")
    }

    pub fn person_experiences_path(&self) -> PathBuf {
        self.person_data_dir()
            .join(format!("experiences_{}.typ", self.lang))
    }

    pub fn person_image_path(&self) -> PathBuf {
        self.person_data_dir().join("profile.png")
    }
}
