use std::{ffi::OsStr, path::{absolute, Path, PathBuf}};
use anyhow::{anyhow, Context, Result};
use tempfile::Builder;

/// takes a local target path and returns the path to the temporary root dir/file
/// assumes that a provided path is valid
pub fn generate_temp_entry_point(local_target_path: &str) -> Result<PathBuf> {
    let path = Path::new(&local_target_path);
    assert!(path.exists());
    let path = absolute(&path).context("making the local path absolute")?;

    let parent = path.parent();
    if parent.is_none() { return Err(anyhow!("local target path does not have a parent")); }
    let parent = parent.unwrap();

    let child = path.file_name();
    if child.is_none() { return Err(anyhow!("local target path does not have a child")); }
    let child = child.unwrap();

    if path.is_file() {
        let temp_file_path = generate_temp_file(child, parent)?;
        Ok(temp_file_path)
    } else {
        let temp_dir_path = generate_temp_dir(child, parent)?;
        Ok(temp_dir_path)
    }
}

fn generate_temp_file(child: &OsStr, path: &Path) -> Result<PathBuf> {
        let temp_file = Builder::new()
            .prefix(child)
            .rand_bytes(6)
            .tempfile_in(path)
            .context("creating temp file")?;

        let (_temp_file, temp_file_path) = temp_file.keep().context("keeping temp file")?;
        Ok(temp_file_path)
}

fn generate_temp_dir(child: &OsStr, path: &Path) -> Result<PathBuf> {
        let temp_dir = Builder::new()
            .prefix(child)
            .rand_bytes(6)
            .tempdir_in(path)
            .context("creating temp dir")?;

        let temp_dir_path = temp_dir.into_path();
        Ok(temp_dir_path)
}
