extern crate colored;
extern crate serde;

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
use serde::{Deserialize, Serialize};
use serde_json;

// Local modules
use errors::IronSSGError;
use page_manifest::PageManifest;

#[derive(Deserialize, Serialize, Debug)]
pub struct IronSSGConfig {
    pub dev: Option<bool>,
    pub verbose: Option<bool>,
    pub clean: Option<bool>,
    pub dist: Option<String>,
    #[serde(rename = "static_assets")]
    pub static_assets: Option<Vec<String>>,
    pub authors: Vec<String>,
    pub name: String,
    pub version: String,
    pub page: Vec<IronSSGPage>,
}

impl Default for IronSSGConfig {
    fn default() -> Self {
        Self {
            dev: Some(true),
            verbose: Some(true),
            clean: Some(false),
            dist: Some("dist".to_string()),
            static_assets: None,
            authors: Vec::new(),
            name: "IronSSG Website".to_string(),
            version: "0.1.0".to_string(),
            page: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IronSSGPage {
    controller: Option<String>,
    path: Option<String>,
    slug: String,
    title: String,
    description: Option<String>,
    view: String,
    components: Option<Vec<String>>,
    model: Option<String>,
}

impl Default for IronSSGPage {
    fn default() -> Self {
        Self {
            controller: None,
            path: None,
            slug: "index".to_string(),
            title: "".to_string(),
            description: None,
            view: "".to_string(),
            components: None,
            model: None,
        }
    }
}

pub struct IronSSG<'a> {
    pub manifest: Vec<PageManifest>,
    pub config: IronSSGConfig,
    pub handlebars: Handlebars<'a>,
}

// Constructor
impl<'a> IronSSG<'a> {
    pub fn new(config: IronSSGConfig) -> Result<Self, IronSSGError> {
        let handlebars = Handlebars::new();
        let manifest = Vec::new();

        let mut ssg = Self {
            manifest,
            config,
            handlebars,
        };

        // Separate the collection of pages and their processing into two phases
        let pages = ssg.config.page.clone();
        for page in &pages {
            // println!("Controller: {:?}", page.controller);
            // println!("Components: {:?}", page.components);

            if let Err(e) = ssg.build_page_manifest(&page) {
                let page_error_message = format!("Failed to create page: {:?}", e).red();
                eprintln!("{}", page_error_message);
            }
        }

        // ssg.serialize_manifest()?;

        Ok(ssg)
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
                    println!(
                        "Copied static_assets folder: '{}' to: '{}'",
                        &public_folder.blue(),
                        &dist_folder.green()
                    );
                } else {
                    let static_folder_error = format!(
                        "{} static_assets folder '{}' does not exist, skipping.",
                        "Note: ".red(),
                        &public_folder.red()
                    );
                    eprintln!("{}", &static_folder_error.bright_black());
                }
            }
        } else {
            println!("No 'static_assets' folders specified in config.");
        }

        Ok(())
    }

    pub fn generate_page(&self, manifest: PageManifest) -> Result<(), IronSSGError> {
        let generating_message =
            format!("Generating: {:?}", manifest.view_file_path).bright_black();
        println!("{}", generating_message);
        // Create the output directory if it doesn't exist
        if !Path::new(&manifest.dist_path).exists() {
            create_dir_all(&manifest.dist_path)?;
        }
        let output = self
            .handlebars
            .render_template(&manifest.view, &manifest.model)?;
        fs::write(&manifest.dist_file_path, output)?;
        println!("Generated:  {}", manifest.dist_file_path.green());
        Ok(())
    }

    pub fn generate(&self) -> Result<(), IronSSGError> {
        if self.config.clean.unwrap_or_default() {
            // Remove existing 'dist' folder
            if let Err(e) = fs::remove_dir_all("dist") {
                eprintln!("Warning: Couldn't remove the 'dist' directory. {}", e);
            }
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
