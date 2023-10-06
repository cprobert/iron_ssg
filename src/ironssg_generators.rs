extern crate colored;

// Standard libraries
use std::{error::Error, fs, fs::create_dir_all, path::Path, result::Result};

// Third-party libraries
use colored::*;

// Local modules
use crate::iron_ssg::errors::IronSSGError;
use crate::iron_ssg::file_utils;
use crate::iron_ssg::ironssg_page::IronSSGPage;

// Generators
impl<'a> crate::IronSSG {
    pub fn copy_public_folders(&self) -> Result<(), Box<dyn Error>> {
        let dist_folder = self
            .config
            .output
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
                    let static_folder_message = format!(
                        "{} All static_assets in '{}' copied to '{}'",
                        "Setup: ".yellow(),
                        &public_folder.blue(),
                        &dist_folder.blue()
                    );
                    println!("{}", static_folder_message.bright_black());
                } else {
                    let static_folder_error = format!(
                        "{} static_assets folder '{}' does not exist, skipping.",
                        "Warning: ".bright_magenta(),
                        &public_folder.red()
                    );
                    eprintln!("{}", &static_folder_error);
                }
            }
        } else {
            println!("No 'static_assets' folders specified in config.");
        }

        Ok(())
    }

    pub fn generate_page(&self, manifest: IronSSGPage) -> Result<(), IronSSGError> {
        if self.config.verbose.unwrap_or_default() {
            let generating_message =
                format!("Generating: {:?}", manifest.view_file_path).bright_black();
            println!("{}", generating_message);
        }

        // Create the output directory if it doesn't exist
        if !Path::new(&manifest.dist_path).exists() {
            create_dir_all(&manifest.dist_path)?;
        }

        // Step 2: Parse JSON to Rust data structure
        // let parsed_json: Value =
        //     serde_json::from_str(&manifest.model).expect("Failed to parse JSON");

        // let mut context = tera::Context::new();
        // context.insert("data", &manifest.model);

        let output = self.tera.render(
            &manifest.view_file_path,
            &tera::Context::from_serialize(&manifest.model)?,
        )?;

        fs::write(&manifest.dist_file_path, output)?;

        println!(
            "{} {}",
            "Generated:".bright_black(),
            manifest.dist_file_path.green()
        );
        Ok(())
    }

    pub fn generate(&mut self) -> Result<(), IronSSGError> {
        if self.config.clean.unwrap_or_default() {
            // Remove existing 'dist' folder

            let dist = self
                .config
                .output
                .clone()
                .unwrap_or_else(|| "dist".to_string());

            let clean_message =
                format!("Warning: Couldn't remove the '{}' directory.", &dist).red();

            if let Err(e) = fs::remove_dir_all(&dist) {
                eprintln!("{} {}", clean_message, e);
            }
        }

        let pages = self.config.page.clone();
        for page in &pages {
            // println!("Controller: {:?}", page.controller);
            // println!("Components: {:?}", page.components);

            if let Err(e) = self.build_page_manifest(&page) {
                let page_error_message = format!("{:?}", e).red();
                eprintln!(
                    "{}{}",
                    "Failed to create page manifest: \n".bright_black(),
                    page_error_message
                );
            }
        }

        if self.config.logging.unwrap_or_default() {
            file_utils::serialize_manifest(&self.manifest)?;
        }

        self.copy_public_folders()?;

        for manifest in &self.manifest {
            self.generate_page(manifest.clone())?;
        }
        Ok(())
    }
}
