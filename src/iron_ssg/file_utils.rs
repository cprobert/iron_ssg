use crate::iron_ssg::{config::IronSSGConfig, errors::IronSSGError, page_manifest::PageManifest};

use std::{error::Error, fs, fs::File, io, io::Read, io::Write, path::Path};

pub fn copy_folder_contents(dir: &Path, target_dir: &Path) -> io::Result<()> {
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

// pub fn read_view_file(view_file_path: &str) -> Result<String, IronSSGError> {
//     // Try to open the file, map any error to your custom type
//     let mut view_file = fs::File::open(view_file_path).map_err(|e| {
//         IronSSGError::FileError(io::Error::new(
//             e.kind(),
//             format!("Failed to open view file: {}", view_file_path),
//         ))
//     })?;

//     let mut view: String = String::new();

//     // Try to read the file, map any error to your custom type
//     view_file.read_to_string(&mut view).map_err(|e| {
//         IronSSGError::FileError(io::Error::new(
//             e.kind(),
//             "Failed to read view file into string",
//         ))
//     })?;

//     Ok(view)
// }

pub fn log_config(
    config_path: &String,
    config: &IronSSGConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize the config to a JSON string
    let config_json = serde_json::to_string_pretty(config)?;

    // Ensure the _logs directory exists
    if !Path::new("_logs").exists() {
        std::fs::create_dir("_logs")?;
    }

    // Open the file for writing
    let file_path = format!("_logs/{}.json", config_path);
    let mut file = File::create(file_path)?;
    // Write the JSON string to the file
    file.write_all(config_json.as_bytes())?;

    Ok(())
}

#[allow(warnings)]
pub fn serialize_manifest(manifest: &Vec<PageManifest>) -> Result<(), Box<dyn Error>> {
    let serialized_manifest = serde_json::to_string(&manifest)?;
    let mut file = File::create("_logs/manifest.json")?;
    file.write_all(serialized_manifest.as_bytes())?;
    Ok(())
}
