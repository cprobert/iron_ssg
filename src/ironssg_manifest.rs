// Third-party libraries
use chrono::{Datelike, Utc};
use json::{self, JsonValue};
use serde_json;

// Standard libraries
use std::{error::Error, fs::File, io::Read, result::Result};

// Local modules
use crate::iron_ssg::config::IronSSGPage;
use crate::iron_ssg::errors::IronSSGError;
use crate::iron_ssg::page_manifest::PageManifest;

// Build manifest
impl<'a> crate::IronSSG {
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
