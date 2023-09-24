use std::fs;
use std::io;
use std::path::Path;

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
