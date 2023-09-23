use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{self, Value};
use std::fs;
use std::fs::create_dir_all;
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
}

pub struct IronSSG<'a> {
    pub manifest: Vec<PageManifest>,
    pub config: IronSSGConfig,
    pub handlebars: Handlebars<'a>,
}

#[derive(Serialize, Clone)]
pub struct PageManifest {
    pub view: String,
    pub model: String,
    pub controller: String,
    pub path: String,
    pub slug: String,
    pub title: String,
    pub description: String,
}

impl<'a> IronSSG<'a> {
    pub fn new(config: Option<IronSSGConfig>) -> Result<Self, IronSSGError> {
        let default_config = IronSSGConfig {
            dev: false,
            verbose: false,
        };

        let config = config.unwrap_or_else(|| {
            eprintln!("Warning: No config provided. Using default settings.");
            default_config
        });

        let handlebars = Handlebars::new();

        // Remove existing 'dist' folder
        if let Err(e) = fs::remove_dir_all("dist") {
            if config.verbose {
                eprintln!("Warning: Couldn't remove the 'dist' directory. {}", e);
            }
        }

        Ok(Self {
            manifest: Vec::new(),
            config,
            handlebars,
        })
    }

    pub fn page(&mut self, json: &str) -> Result<(), IronSSGError> {
        let v: Value = serde_json::from_str(json)?;

        let manifest = PageManifest {
            title: v["title"].as_str().unwrap_or("").to_string(),
            view: v["view"].as_str().unwrap_or("").to_string(),
            model: v["model"].as_str().unwrap_or("").to_string(),
            controller: v["controller"].as_str().unwrap_or("").to_string(),
            path: v["path"].as_str().unwrap_or("").to_string(),
            slug: v["slug"].as_str().unwrap_or("index").to_string(),
            description: v["description"].as_str().unwrap_or("").to_string(),
        };

        self.manifest.push(manifest);
        Ok(())
    }

    pub fn generate_page(&self, manifest: PageManifest) -> Result<(), IronSSGError> {
        println!("Generating: {:?}", manifest.view);

        let mut view_content: String = String::new();
        let mut file: fs::File = fs::File::open(&manifest.view)?;
        file.read_to_string(&mut view_content)?;

        let output = self.handlebars.render_template(&view_content, &manifest)?;

        // Prepare the output directory and file name
        let dir_path = if !manifest.path.is_empty() {
            format!("dist/{}", manifest.path)
        } else {
            "dist".to_string()
        };
        let file_path = format!("{}/{}.html", dir_path, manifest.slug);

        // Create the output directory if it doesn't exist
        if !Path::new(&dir_path).exists() {
            create_dir_all(&dir_path)?;
        }

        // Write to the output file
        fs::write(&file_path, output)?;
        println!("Generated:  {}", file_path);

        Ok(())
    }

    pub fn generate(&self) -> Result<(), IronSSGError> {
        for manifest in &self.manifest {
            self.generate_page(manifest.clone())?;
        }
        Ok(())
    }
}
