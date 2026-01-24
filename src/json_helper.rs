use crate::types::DuplicatesRegistry;
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

pub fn read_from_json(json_path: &str) -> DuplicatesRegistry {
    let mut file = File::open(json_path).expect("Failed to open JSON file");
    let mut json = String::new();
    file.read_to_string(&mut json)
        .expect("Failed to read JSON file");

    let registry: DuplicatesRegistry =
        serde_json::from_str(&json).expect("Failed to deserialize JSON");

    println!(
        "Loaded {} duplicate records from {json_path}",
        registry.duplicates.len()
    );

    registry
}

pub fn save_to_json(registry: DuplicatesRegistry, json_path: &str) {
    let json = serde_json::to_string_pretty(&registry).expect("Failed to serialize to JSON");

    let path = PathBuf::from(json_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create directories");
    }

    let mut file = File::create(json_path).expect(&format!("Failed to create {json_path}"));
    file.write_all(json.as_bytes())
        .expect(&format!("Failed to write to {json_path}"));

    println!("Saved to {json_path}");
}
