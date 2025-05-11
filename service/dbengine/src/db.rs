use crate::sandbox::compiler;
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, create_dir_all, read_dir, OpenOptions};
use std::io::ErrorKind::InvalidInput;
use std::io::Write;
use std::os::fd::BorrowedFd;
use std::path::{absolute, Path, PathBuf};
use serde_json::value::RawValue;

lazy_static! {
    static ref base_data: PathBuf = absolute(Path::new("./data")).unwrap();
    static ref base_queries: PathBuf = absolute(Path::new("./queries")).unwrap();
}

pub fn dir(database: &str) -> PathBuf {
    base_data.join(database)
}

pub fn query_object(code_ref: &str) -> PathBuf {
    base_queries.join(format!("{}.so", code_ref))
}

pub fn list_users() -> std::io::Result<Vec<String>> {
    println!("listing users");
    let users = dir("_").join("users.ndjson");
    // todo can we get some sort of file lock here that prevents new user registration?
    let contents = fs::read_to_string(users);
    let contents = contents.expect("Could not open users file");
    let contents = contents.lines().map(String::from);
        //.collect();

    let mut vec: Vec<String> = Vec::new();
    for line in contents {
        let v: Value = serde_json::from_str(&line)?;

        println!("User data: {:?}", v);
        vec.push(v["username"].to_string());
    }
    // for line in lines {
    // }
    Ok(vec)
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

pub async fn prepare_query(code: &str) -> std::io::Result<String> {
    let code_ref = hex::encode(Sha256::digest(code.as_bytes()));
    let obj = query_object(&code_ref);

    if !obj.exists() {
        create_dir_all(base_queries.as_path())?;
        if let Err(e) = compiler::compile(code, obj).await {
            return Err(std::io::Error::new(InvalidInput, format!("compile error {}", e)));
        }
    }

    Ok(code_ref)
}

pub async fn execute_query(database: &str, code_ref: &str, query: &str, param: &RawValue, output: BorrowedFd<'_>) -> std::io::Result<()> {
    let obj = query_object(code_ref);
    let dir = dir(database);
    create_dir_all(&dir)?;
    let param = serde_json::to_string(param)?;
    if let Err(e) = compiler::run(obj, dir, query, param.as_str(), output).await {
        return Err(std::io::Error::new(InvalidInput, format!("run error {}", e)));
    }
    Ok(())
}
