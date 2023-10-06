extern crate colored;
extern crate serde;

mod ironssg_generators;
mod ironssg_manifest;

mod iron_ssg {
    pub mod config;
    pub mod errors;
    pub mod file_utils;
    pub mod page_manifest;
}

// Standard libraries
use std::{error::Error, fs::File, io::Read, result::Result};

// Third-party libraries
use colored::*;
use tera::Tera;

// Local modules
use crate::iron_ssg::config::IronSSGConfig;
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
