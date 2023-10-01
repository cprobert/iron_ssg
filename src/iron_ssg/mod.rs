extern crate colored;
extern crate serde;

pub mod config;
pub mod errors;
pub mod file_utils;
pub mod page_manifest;

// Standard libraries
use std::{
    error::Error, fs, fs::create_dir_all, fs::File, io::Read, io::Write, path::Path, result::Result,
};

// Third-party libraries
use chrono::{Datelike, Utc};
use colored::*;
use handlebars::Handlebars;
use json::{self, JsonValue};

use serde_json;

// Local modules
use config::{IronSSGConfig, IronSSGPage};
use errors::IronSSGError;
use page_manifest::PageManifest;

fn log_config(
    config_path: &String,
    config: &IronSSGConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize the config to a JSON string
    let config_json = serde_json::to_string_pretty(config)?;

    // Ensure the _logs directory exists
    if !Path::new("_logs").exists() {
        std::fs::create_dir("_logs")?;
    }

    // Open the file for writing
    let file_path = format!("_logs/{}.json", config_path);
    let mut file = File::create(file_path)?;
    // Write the JSON string to the file
    file.write_all(config_json.as_bytes())?;

    Ok(())
}

pub struct IronSSG<'a> {
    pub manifest: Vec<PageManifest>,
    pub config: IronSSGConfig,
    pub handlebars: Handlebars<'a>,
}

// Constructor
impl<'a> IronSSG<'a> {
    pub fn new(config_path: &str) -> Result<Self, IronSSGError> {
        let init_msg = format!(
            "{} IronSSG with config: {}",
            "Initializing: ".yellow(),
            config_path.blue()
        );
        println!("{}", init_msg.bright_black());

        let mut file = File::open(config_path)
            .map_err(|_| IronSSGError::CustomError("Unable to open config".to_string()))?;
        let mut data = String::new();
        file.read_to_string(&mut data)
            .map_err(|_| IronSSGError::CustomError("Unable to read config.toml".to_string()))?;

        let config: IronSSGConfig = toml::from_str(&data)
            .map_err(|e| IronSSGError::CustomError(format!("Deserialization error: {}", e)))?;

        if config.logging.unwrap_or_default() {
            log_config(&config_path.to_string(), &config)?;
        }

        let handlebars = Handlebars::new();
        let manifest = Vec::new();

        Ok(Self {
            manifest,
            config,
            handlebars,
        })
    }
}

// Build manifest
impl<'a> IronSSG<'a> {
    pub fn build_page_manifest(&mut self, page: &IronSSGPage) -> Result<(), Box<dyn Error>> {
        // Check mandatory fields
        if page.title.is_empty() {
            return Err(Box::new(IronSSGError::CustomError(
                "Missing 'title' field".to_string(),
            )));
        }
        if page.view.is_empty() {
            return Err(Box::new(IronSSGError::CustomError(
                "Missing 'view' field".to_string(),
            )));
        }

        // Prepare the output directory and file name
        let dist_path = if !page.path.as_ref().unwrap_or(&"".to_string()).is_empty()
            && page.path.as_ref().unwrap_or(&"".to_string()) != "/"
        {
            format!(
                "{}/{}",
                self.config.dist.as_ref().unwrap_or(&"dist".to_string()),
                page.path
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .trim_end_matches('/')
            )
        } else {
            self.config
                .dist
                .as_ref()
                .unwrap_or(&"dist".to_string())
                .to_string()
        };

        let dist_file_path = format!("{}/{}.html", dist_path, page.slug);

        // Get the view file contents
        let view: String = file_utils::read_view_file(&page.view)?;

        // Initialize model as an empty JSON object
        let mut model: json::JsonValue = json::object! {};

        // Check if the file has a .json extension
        if let Some(model_path) = &page.model {
            if model_path.ends_with(".json") {
                // Open the file
                let mut file = File::open(model_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                // Parse the JSON into a JsonValue
                let parsed_json = json::parse(&contents)?;
                model = parsed_json;
            }
        }

        // Add nested properties
        let mut metadata = JsonValue::new_object();
        metadata["title"] = json::JsonValue::String(page.title.to_string());
        metadata["description"] =
            json::JsonValue::String(page.description.clone().unwrap_or_default());
        metadata["author"] = JsonValue::String("Courtenay Probert".to_string());
        let current_year = Utc::now().year();
        metadata["year"] = JsonValue::Number(current_year.into());
        model["metadata"] = metadata;

        // This is a hack to get a Serializable object for handlebars
        // json::object is much easier to work with, but it's not Serializable
        let model_str = model.dump();
        let model_serializable: serde_json::Value = serde_json::from_str(&model_str).unwrap();

        let manifest = PageManifest {
            title: page.title.to_string(),
            view_file_path: page.view.to_string(),
            model_file_path: page.model.clone().unwrap_or_default(),
            dist_path,
            dist_file_path,
            view,
            model: model_serializable,
        };

        self.manifest.push(manifest);

        Ok(())
    }
}

// Generators
impl<'a> IronSSG<'a> {
    pub fn copy_public_folders(&self) -> Result<(), Box<dyn Error>> {
        let dist_folder = self
            .config
            .dist
            .clone()
            .unwrap_or_else(|| "dist".to_string());

        let dist_folder_path = Path::new(&dist_folder);
        if !dist_folder_path.exists() {
            fs::create_dir_all(&dist_folder_path)?;
        }

        if let Some(static_assets) = &self.config.static_assets {
            for public_folder in static_assets {
                let public_folder_path = Path::new(&public_folder);
                if public_folder_path.exists() {
                    file_utils::copy_folder_contents(&public_folder_path, &dist_folder_path)?;
                    let static_folder_message = format!(
                        "{} All static_assets in '{}' copied to '{}'",
                        "Setup: ".yellow(),
                        &public_folder.blue(),
                        &dist_folder.blue()
                    );
                    println!("{}", static_folder_message.bright_black());
                } else {
                    let static_folder_error = format!(
                        "{} static_assets folder '{}' does not exist, skipping.",
                        "Warning: ".bright_magenta(),
                        &public_folder.red()
                    );
                    eprintln!("{}", &static_folder_error);
                }
            }
        } else {
            println!("No 'static_assets' folders specified in config.");
        }

        Ok(())
    }

    pub fn generate_page(&self, manifest: PageManifest) -> Result<(), IronSSGError> {
        if self.config.verbose.unwrap_or_default() {
            let generating_message =
                format!("Generating: {:?}", manifest.view_file_path).bright_black();
            println!("{}", generating_message);
        }

        // Create the output directory if it doesn't exist
        if !Path::new(&manifest.dist_path).exists() {
            create_dir_all(&manifest.dist_path)?;
        }
        let output = self
            .handlebars
            .render_template(&manifest.view, &manifest.model)?;
        fs::write(&manifest.dist_file_path, output)?;
        println!(
            "{} {}",
            "Generated:".bright_black(),
            manifest.dist_file_path.green()
        );
        Ok(())
    }

    pub fn generate(&mut self) -> Result<(), IronSSGError> {
        if self.config.clean.unwrap_or_default() {
            // Remove existing 'dist' folder
            if let Err(e) = fs::remove_dir_all("dist") {
                eprintln!("Warning: Couldn't remove the 'dist' directory. {}", e);
            }
        }

        let pages = self.config.page.clone();
        for page in &pages {
            // println!("Controller: {:?}", page.controller);
            // println!("Components: {:?}", page.components);

            if let Err(e) = self.build_page_manifest(&page) {
                let page_error_message = format!("Failed to create page: {:?}", e).red();
                eprintln!("{}", page_error_message);
            }
        }

        if self.config.logging.unwrap_or_default() {
            self.serialize_manifest()?;
        }

        self.copy_public_folders()?;

        for manifest in &self.manifest {
            self.generate_page(manifest.clone())?;
        }
        Ok(())
    }

    #[allow(warnings)]
    pub fn serialize_manifest(&self) -> Result<(), Box<dyn Error>> {
        let serialized_manifest = serde_json::to_string(&self.manifest)?;
        let mut file = File::create("_logs/manifest.json")?;
        file.write_all(serialized_manifest.as_bytes())?;
        Ok(())
    }
}
