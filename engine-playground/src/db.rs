use lazy_static::lazy_static;
use serde::Serialize;
use std::fs::{create_dir_all, read_dir, OpenOptions};
use std::io::Write;
use std::path::{absolute, Path, PathBuf};

lazy_static! {
    static ref BASE: PathBuf = absolute(Path::new("./data")).unwrap();
}

pub fn dir(database: &str) -> PathBuf {
    BASE.join(database)
}

pub fn list_collections(database: &str) -> std::io::Result<Vec<String>> {
    let dir = dir(database);
    create_dir_all(&dir)?;

    let mut vec: Vec<String> = Vec::new();
    for child in read_dir(dir)? {
        let child = child?;
        if child.file_type()?.is_file() {
            let path = child.path();
            if path.extension().map_or(false, |ext| ext == "ndjson") {
                vec.push(path.file_stem().unwrap().to_str().unwrap().to_string());
            }
        }
    }

    Ok(vec)
}

pub fn append<T: Serialize>(database: &str, collection: &str, data: &T) -> std::io::Result<()> {
    let dir = dir(database);
    create_dir_all(&dir)?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(dir.join(format!("{}.ndjson", collection)))?;
    serde_json::to_writer(&mut file, data)?;
    file.write_all("\n".as_bytes())
}
