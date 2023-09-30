pub mod errors;
pub mod file_utils;
pub mod page_manifest;

// Standard libraries
use std::{error::Error, fs, fs::create_dir_all, fs::File, io::Read, path::Path, result::Result};

// Third-party libraries
use chrono::{Datelike, Utc};
use handlebars::Handlebars;
use json::{self, JsonValue};
use serde::Deserialize;
use serde_json;

// Local modules
use errors::IronSSGError;
use page_manifest::PageManifest;

#[derive(Deserialize)]
pub struct IronSSGConfig {
    pub dev: Option<bool>,
    pub verbose: Option<bool>,
    pub clean: Option<bool>,
    pub dist: Option<String>,
    pub public: Option<String>,
    pub authors: Vec<String>,
    pub name: String,
    pub version: String,
    pub pages: Vec<IronSSGPage>,
    pub static_files: Vec<String>,
}

impl Default for IronSSGConfig {
    fn default() -> Self {
        Self {
            dev: Some(true),
            verbose: Some(true),
            clean: Some(false),
            dist: Some("dist".to_string()),
            public: Some("public".to_string()),
            authors: Vec::new(),
            name: "Terra App".to_string(),
            version: "0.1.0".to_string(),
            pages: Vec::new(),
            static_files: Vec::new(),
        }
    }
}

#[derive(Deserialize, Clone)]
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
        let pages = ssg.config.pages.clone();
        for page in &pages {
            if let Err(e) = ssg.page(&page) {
                eprintln!("Failed to create page: {:?}", e);
            }
        }

        Ok(ssg)
    }

    pub fn page(&mut self, page: &IronSSGPage) -> Result<(), Box<dyn Error>> {
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

    pub fn generate_page(&self, manifest: PageManifest) -> Result<(), IronSSGError> {
        println!("Generating: {:?}", manifest.view_file_path);
        // Create the output directory if it doesn't exist
        if !Path::new(&manifest.dist_path).exists() {
            create_dir_all(&manifest.dist_path)?;
        }
        let output = self
            .handlebars
            .render_template(&manifest.view, &manifest.model)?;
        fs::write(&manifest.dist_file_path, output)?;
        println!("Generated:  {}", manifest.dist_file_path);
        Ok(())
    }

    pub fn generate(&self) -> Result<(), IronSSGError> {
        if self.config.clean.unwrap_or_default() {
            // Remove existing 'dist' folder
            if let Err(e) = fs::remove_dir_all("dist") {
                eprintln!("Warning: Couldn't remove the 'dist' directory. {}", e);
            }
        }

        // Copying files from `self.config.public` into `dist` folder
        let public_folder = self.config.public.clone().unwrap_or_default();
        println!("Copying public folder: {}", &public_folder);
        let dist_folder = self.config.dist.clone().unwrap_or_default();
        println!("To: {}", &dist_folder);

        let public_folder_path = Path::new(&public_folder);
        let dist_folder_path = Path::new(&dist_folder);

        if !dist_folder_path.exists() {
            fs::create_dir_all(&dist_folder_path)?;
        }

        file_utils::copy_folder_contents(&public_folder_path, &dist_folder_path)?;

        for manifest in &self.manifest {
            self.generate_page(manifest.clone())?;
        }
        Ok(())
    }
}
