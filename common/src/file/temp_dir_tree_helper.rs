use std::{
    ffi::OsStr,
    fs::create_dir_all,
    path::{absolute, Path, PathBuf}
};
use anyhow::{anyhow, Context, Result};
use tempfile::Builder;
use super::FileMeta;

/// takes a local target path and generates the whole temp tree
/// returns the path to the temporary root dir/file
/// assumes that a provided path is valid
pub fn generate_temp_entry_point(local_target_path: &str, remote_target_path: &str, checksums: &Vec<FileMeta>) -> Result<PathBuf> {
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
        let temp_file_path = generate_temp_file(child, parent, 6)?;
        Ok(temp_file_path)
    } else {
        let temp_dir_path = generate_temp_dir(child, parent)?;
        generate_temp_tree(&temp_dir_path, remote_target_path, checksums)
            .context("generating temp dir tree")?;
        Ok(temp_dir_path)
    }
}

fn generate_temp_tree(entry_point: &PathBuf, remote_target_path: &str, checksums: &Vec<FileMeta>) -> Result<()> {
    let remote_path = Path::new(&remote_target_path);

    for file in checksums {
        let file_local_path = file.path.strip_prefix(remote_path.to_str().unwrap());
        if file_local_path.is_none() { return Err(anyhow!("Remote file path malformed")); }
        let file_local_path = file_local_path.unwrap();

        // remote path can have trailing slash when syncing a dirctory
        let file_local_path = if file_local_path.starts_with("/") { file_local_path.strip_prefix("/").unwrap() } else { file_local_path };

        let mut local_path = PathBuf::from(entry_point);
        local_path.push(file_local_path);

        let file_parent = local_path.parent();
        if file_parent.is_none() { return Err(anyhow!("Remote file path malformed")); }
        let file_parent = file_parent.unwrap();

        create_dir_all(file_parent).context("creating dir tree")?;

        let file_child = local_path.file_name();
        if file_child.is_none() {
            return Err(anyhow!("Remote file path malformed"));
        }
        let file_child = file_child.unwrap();

        generate_temp_file(file_child, &file_parent, 0)?;
    }
    Ok(())
}

fn generate_temp_file(child: &OsStr, path: &Path, number_of_bytes: usize) -> Result<PathBuf> {
        let temp_file = Builder::new()
            .prefix(child)
            .rand_bytes(number_of_bytes)
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
