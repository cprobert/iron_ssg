extern crate colored;
extern crate serde;

mod ironssg_generators;
mod iron_ssg {
    pub mod config;
    pub mod errors;
    pub mod file_utils;
    pub mod page_manifest;
}

// Standard libraries
use std::{error::Error, fs::File, io::Read, result::Result};
// fs::create_dir_all, fs, , io::Read, path::Path

// Third-party libraries
use chrono::{Datelike, Utc};
use colored::*;
use json::{self, JsonValue};
use serde_json;
use tera::Tera;

// Local modules
use crate::iron_ssg::config::{IronSSGConfig, IronSSGPage};
use crate::iron_ssg::errors::IronSSGError;
use crate::iron_ssg::file_utils;
use crate::iron_ssg::page_manifest::PageManifest;

pub struct IronSSG {
    pub manifest: Vec<PageManifest>,
    pub config: IronSSGConfig,
    pub tera: Tera,
}

// Constructor
impl<'a> IronSSG {
    pub fn new(config_path: &str) -> Result<Self, IronSSGError> {
        let init_msg = format!(
            "{} IronSSG with config: {}",
            "Initializing: ".yellow(),
            config_path.blue()
        );
        println!("{}", init_msg.bright_black());

        // Lets map the config file to a struct
        let mut file = File::open(config_path)
            .map_err(|_| IronSSGError::CustomError("Unable to open config".to_string()))?;
        let mut data = String::new();
        file.read_to_string(&mut data)
            .map_err(|_| IronSSGError::CustomError("Unable to read config.toml".to_string()))?;

        let config: IronSSGConfig = toml::from_str(&data)
            .map_err(|e| IronSSGError::CustomError(format!("Deserialization error: {}", e)))?;

        if config.logging.unwrap_or_default() {
            file_utils::log_config(&config_path.to_string(), &config)?;
        }

        // Let's initialize Tera and load up the templates
        // this is where we might see an error if the templates have any issues
        // Tera is very unforgiving
        let template_folder = config.template_folder.clone();
        let template_glob = format!("{}/**/*.{{tera,html,md}}", template_folder);

        let tera_result = Tera::new(&template_glob);

        let tera = match tera_result {
            Ok(t) => t,
            Err(e) => {
                let tera_error_message = format!("{:?}", e).bright_black();
                eprintln!(
                    "{} {}",
                    "Failed to initialize Tera templates:\n".red(),
                    tera_error_message
                );
                let mut cause = e.source();
                while let Some(err) = cause {
                    eprintln!("{} {}", "Caused by:".yellow(), err);
                    cause = err.source();
                }
                return Err(IronSSGError::CustomError(
                    "Failed to initialize Tera templates".to_string(),
                ));
            }
        };

        let manifest = Vec::new();

        Ok(Self {
            manifest,
            config,
            tera,
        })
    }
}

// Build manifest
impl<'a> IronSSG {
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
                self.config.output.as_ref().unwrap_or(&"dist".to_string()),
                page.path
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .trim_end_matches('/')
            )
        } else {
            self.config
                .output
                .as_ref()
                .unwrap_or(&"dist".to_string())
                .to_string()
        };

        let dist_file_path = format!("{}/{}.html", dist_path, page.slug);

        // // Get the view file contents
        // let view: String = file_utils::read_view_file(&page.view)?;

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

        // This is a hack to get a Serializable object for the view
        // json::object is much easier to work with, but it's not Serializable
        let model_str = model.dump();
        let model_serializable: serde_json::Value = serde_json::from_str(&model_str).unwrap();

        let manifest = PageManifest {
            title: page.title.to_string(),
            view_file_path: page.view.to_string(),
            model_file_path: page.model.clone().unwrap_or_default(),
            dist_path,
            dist_file_path,
            model: model_serializable,
        };

        self.manifest.push(manifest);

        Ok(())
    }
}
