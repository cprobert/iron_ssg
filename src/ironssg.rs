use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{self, Value};
use std::error::Error;
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
// use std::io::prelude::*;
use std::io::{self, Read};
use std::path::Path;
use std::result::Result;

#[derive(Debug)]
pub enum IronSSGError {
    InvalidJSON(serde_json::Error),
    FileError(io::Error),
    RenderError(handlebars::RenderError),
}

impl From<serde_json::Error> for IronSSGError {
    fn from(err: serde_json::Error) -> Self {
        IronSSGError::InvalidJSON(err)
    }
}

impl From<io::Error> for IronSSGError {
    fn from(err: io::Error) -> Self {
        IronSSGError::FileError(err)
    }
}

impl From<handlebars::RenderError> for IronSSGError {
    fn from(err: handlebars::RenderError) -> Self {
        IronSSGError::RenderError(err)
    }
}

pub struct IronSSGConfig {
    pub dev: bool,
    pub verbose: bool,
    pub clean: bool,
}

pub struct IronSSG<'a> {
    pub manifest: Vec<PageManifest>,
    pub config: IronSSGConfig,
    pub handlebars: Handlebars<'a>,
}

#[derive(Serialize, Clone)]
pub struct PageManifest {
    pub title: String,
    pub description: String,
    pub view_file_path: String,
    pub model_file_path: String,
    pub output_file_path: String,
    pub view: String,
    pub model: Option<Value>,
}

impl<'a> IronSSG<'a> {
    pub fn new(config: Option<IronSSGConfig>) -> Result<Self, IronSSGError> {
        let default_config = IronSSGConfig {
            dev: false,
            verbose: false,
            clean: false,
        };

        let config = config.unwrap_or_else(|| {
            eprintln!("Warning: No config provided. Using default settings.");
            default_config
        });

        let handlebars = Handlebars::new();

        if config.clean {
            // Remove existing 'dist' folder
            if let Err(e) = fs::remove_dir_all("dist") {
                eprintln!("Warning: Couldn't remove the 'dist' directory. {}", e);
            }
        }

        Ok(Self {
            manifest: Vec::new(),
            config,
            handlebars,
        })
    }

    pub fn page(&mut self, page: &Value) -> Result<(), Box<dyn Error>> {
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
        let dist_file_path = if !path.is_empty() && path != "/" {
            format!("dist/{}", path.trim_end_matches('/'))
        } else {
            "dist".to_string()
        };
        // Create the output directory if it doesn't exist
        if !Path::new(&dist_file_path).exists() {
            create_dir_all(&dist_file_path)?;
        }

        let output_file_path = format!("{}/{}.html", dist_file_path, slug);

        // Get the view
        let mut view: String = String::new();
        let mut file: fs::File = fs::File::open(view_file_path)?;
        file.read_to_string(&mut view)?;

        // Initialize model as None
        let mut model: Option<Value> = None;

        // Check if the file has a .json extension
        if model_file_path.ends_with(".json") {
            // Open the file
            let mut file = File::open(&model_file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // Parse the JSON into a Value
            let parsed_json: Value = serde_json::from_str(&contents)?;
            model = Some(parsed_json);
        }

        let manifest = PageManifest {
            title: title.to_string(),
            view_file_path: view_file_path.to_string(),
            model_file_path: model_file_path.to_string(),
            description,
            output_file_path,
            view,
            model,
        };

        self.manifest.push(manifest);
        Ok(())
    }

    pub fn generate_page(&self, manifest: PageManifest) -> Result<(), IronSSGError> {
        println!("Generating: {:?}", manifest.view_file_path);
        let output = self.handlebars.render_template(&manifest.view, &manifest)?;
        fs::write(&manifest.output_file_path, output)?;
        println!("Generated:  {}", manifest.output_file_path);
        Ok(())
    }

    pub fn generate(&self) -> Result<(), IronSSGError> {
        for manifest in &self.manifest {
            self.generate_page(manifest.clone())?;
        }
        Ok(())
    }
}
