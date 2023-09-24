extern crate fs_extra;
use chrono::{Datelike, Utc};
// use fs_extra::dir::CopyOptions;
use handlebars::Handlebars;
use json;
use json::JsonValue;
use serde_json;
use std::error::Error;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::result::Result;

#[derive(Debug)]
pub enum IronSSGError {
    InvalidJSON(json::Error),
    FileError(io::Error),
    RenderError(handlebars::RenderError),
}

impl fmt::Display for IronSSGError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IronSSGError::InvalidJSON(err) => write!(f, "Invalid JSON: {}", err),
            IronSSGError::FileError(err) => write!(f, "File error: {}", err),
            IronSSGError::RenderError(err) => write!(f, "Rendering error: {}", err),
        }
    }
}

impl StdError for IronSSGError {}

impl From<handlebars::RenderError> for IronSSGError {
    fn from(err: handlebars::RenderError) -> IronSSGError {
        IronSSGError::RenderError(err)
    }
}

impl From<io::Error> for IronSSGError {
    fn from(err: io::Error) -> IronSSGError {
        IronSSGError::FileError(err)
    }
}

impl From<json::Error> for IronSSGError {
    fn from(err: json::Error) -> IronSSGError {
        IronSSGError::InvalidJSON(err)
    }
}

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
#[derive(Clone)]
pub struct PageManifest {
    pub title: String,
    pub view_file_path: String,
    pub model_file_path: String,
    pub dist_path: String,
    pub dist_file_path: String,
    pub view: String,
    pub model: serde_json::Value,
}

fn copy_folder_contents(dir: &Path, target_dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let target_subdir = target_dir.join(path.file_name().unwrap());
                fs::create_dir_all(&target_subdir)?;
                copy_folder_contents(&path, &target_subdir)?;
            } else if path.is_file() {
                let target_file_path = target_dir.join(path.file_name().unwrap());
                fs::copy(&path, &target_file_path)?;
            }
        }
    }
    Ok(())
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
        let mut file = fs::File::open(view_file_path).map_err(|e| {
            IronSSGError::FileError(io::Error::new(
                e.kind(),
                format!("Failed to open view file: {}", view_file_path),
            ))
        })?;

        let mut view: String = String::new();

        // Try to read the file, map any error to your custom type
        file.read_to_string(&mut view).map_err(|e| {
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

        copy_folder_contents(&public_folder_path, &dist_folder_path)?;

        for manifest in &self.manifest {
            self.generate_page(manifest.clone())?;
        }
        Ok(())
    }
}
