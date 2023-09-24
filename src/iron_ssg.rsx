mod errors;
mod file_utils;
mod page_manifest;

// Standard libraries
use std::{
    error::Error,
    fs,
    fs::create_dir_all,
    fs::File,
    io::{self, Read},
    path::Path,
    result::Result,
};

// Third-party libraries
use chrono::{Datelike, Utc};
use handlebars::Handlebars;
use json::{self, JsonValue};
use serde_json;

// Local modules
use errors::IronSSGError;
use page_manifest::PageManifest;

pub struct IronSSGConfig {
    pub dev: bool,
    pub verbose: bool,
    pub clean: bool,
    pub dist: String,
    pub public: String,
}

pub struct IronSSG<'a> {
    pub manifest: Vec<PageManifest>,
    pub config: IronSSGConfig,
    pub handlebars: Handlebars<'a>,
}

impl<'a> IronSSG<'a> {
    pub fn new(config: Option<IronSSGConfig>) -> Result<Self, IronSSGError> {
        let default_config = IronSSGConfig {
            dev: false,
            verbose: false,
            clean: false,
            dist: "dist".to_string(),
            public: "public".to_string(),
        };

        let config = config.unwrap_or_else(|| {
            eprintln!("Warning: No config provided. Using default settings.");
            default_config
        });

        let handlebars = Handlebars::new();

        Ok(Self {
            manifest: Vec::new(),
            config,
            handlebars,
        })
    }

    pub fn page(&mut self, page: &JsonValue) -> Result<(), Box<dyn Error>> {
        // Get the required fields
        let title = page["title"].as_str().ok_or("Missing 'title' field")?;
        let view_file_path = page["view"].as_str().ok_or("Missing 'view' field")?;
        // Get the optional fields
        let model_file_path = page["model"].as_str().unwrap_or("").to_string();
        let description = page["description"].as_str().unwrap_or("").to_string();
        let path = page["path"].as_str().unwrap_or("").to_string();
        let slug = page["slug"].as_str().unwrap_or("index").to_string();
        // let controller = page["controller"]
        //     .as_str()
        //     .ok_or("Missing 'controller' field")?;

        // Prepare the output directory and file name
        let dist_path = if !path.is_empty() && path != "/" {
            format!("{}/{}", self.config.dist, path.trim_end_matches('/'))
        } else {
            self.config.dist.to_string()
        };

        let dist_file_path = format!("{}/{}.html", dist_path, slug);

        // Try to open the file, map any error to your custom type
        let mut view_file = fs::File::open(view_file_path).map_err(|e| {
            IronSSGError::FileError(io::Error::new(
                e.kind(),
                format!("Failed to open view file: {}", view_file_path),
            ))
        })?;

        let mut view: String = String::new();

        // Try to read the file, map any error to your custom type
        view_file.read_to_string(&mut view).map_err(|e| {
            IronSSGError::FileError(io::Error::new(
                e.kind(),
                "Failed to read view file into string",
            ))
        })?;

        // Initialize model as an empty JSON object
        let mut model: json::JsonValue = json::object! {};

        // Check if the file has a .json extension
        if model_file_path.ends_with(".json") {
            // Open the file
            let mut file = File::open(&model_file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // Parse the JSON into a JsonValue
            let parsed_json = json::parse(&contents)?;
            model = parsed_json;
        }
        // // Add new properties to the object
        // model["title"] = json::JsonValue::String(title.to_string());

        // Add nested properties
        let mut metadata = JsonValue::new_object();
        metadata["title"] = json::JsonValue::String(title.to_string());
        metadata["description"] = json::JsonValue::String(description.to_string());
        metadata["author"] = JsonValue::String("Courtenay Probert".to_string());
        let current_year = Utc::now().year();
        metadata["year"] = JsonValue::Number(current_year.into());
        model["metadata"] = metadata;

        // This is a hack to get a Serializable object for handlebars
        // json::object is much easier to work with, but it's not Serializable
        let model_str = model.dump();
        let model_serializable: serde_json::Value = serde_json::from_str(&model_str).unwrap();

        let manifest = PageManifest {
            title: title.to_string(),
            view_file_path: view_file_path.to_string(),
            model_file_path: model_file_path.to_string(),
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
        if self.config.clean {
            // Remove existing 'dist' folder
            if let Err(e) = fs::remove_dir_all("dist") {
                eprintln!("Warning: Couldn't remove the 'dist' directory. {}", e);
            }
        }

        // Copying files from `self.config.public` into `dist` folder
        println!("Copying public folder: {}", &self.config.public);

        let public_folder = &self.config.public;
        let dist_folder = "./dist";
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
