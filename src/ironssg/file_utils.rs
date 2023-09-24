use crate::ironssg::errors::IronSSGError;
// use errors::IronSSGError;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path; // Adjust this import according to where you place your errors module

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

pub fn read_view_file(view_file_path: &str) -> Result<String, IronSSGError> {
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

    Ok(view)
}
