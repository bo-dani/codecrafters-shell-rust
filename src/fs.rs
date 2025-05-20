use anyhow::Result;
use std::env;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

pub fn open(filename: &str, append: bool) -> Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(filename)?;
    Ok(file)
}

pub fn mkdir(path: &str) -> Result<()> {
    let path = Path::new(path);
    let parent = path.parent().unwrap();
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn get_executable_path(cmd: &str) -> Result<Option<PathBuf>> {
    for path in split_path() {
        if !std::fs::exists(&path).unwrap() {
            continue;
        }
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            if cmd == entry.file_name().to_str().unwrap() {
                return Ok(Some(entry.path()));
            }
        }
    }
    Ok(None)
}

pub fn split_path() -> Vec<PathBuf> {
    let path: String = env::var("PATH").expect("PATH environment variable should always exist");
    let mut paths: Vec<PathBuf> = Vec::new();
    for p in path.split(":") {
        paths.push(PathBuf::from(&p));
    }
    paths
}
