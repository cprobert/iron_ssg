use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{self, Value};
use std::fs;
use std::io::{self, Read};
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
            path: v["path"].as_str().unwrap_or("/").to_string(),
            slug: v["slug"].as_str().unwrap_or("index").to_string(),
            description: v["description"].as_str().unwrap_or("").to_string(),
        };

        self.manifest.push(manifest);
        Ok(())
    }

    pub fn generate_page(&self, manifest: PageManifest) -> Result<(), IronSSGError> {
        println!("Generating page: {:?}", manifest.view);

        let mut view_content = String::new();
        let mut file = fs::File::open(&manifest.view)?;
        file.read_to_string(&mut view_content)?;

        let output = self.handlebars.render_template(&view_content, &manifest)?;

        println!("Generated content: {}", output);
        Ok(())
    }

    pub fn generate(&self) -> Result<(), IronSSGError> {
        for manifest in &self.manifest {
            self.generate_page(manifest.clone())?;
        }
        Ok(())
    }
}
