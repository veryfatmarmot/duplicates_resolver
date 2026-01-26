use crate::types::DuplicatesRegistry;
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

pub fn read_from_json(json_path: &str) -> Result<DuplicatesRegistry> {
    let mut file = File::open(json_path).context("Failed to open JSON file")?;
    let mut json = String::new();
    file.read_to_string(&mut json)
        .context("Failed to read JSON file")?;

    let registry: DuplicatesRegistry =
        serde_json::from_str(&json).context("Failed to deserialize JSON")?;

    println!(
        "Loaded {} duplicate records from {json_path}",
        registry.duplicates.len()
    );

    Ok(registry)
}

pub fn save_to_json(registry: DuplicatesRegistry, json_path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(&registry).context("Failed to serialize to JSON")?;

    let path = PathBuf::from(json_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create directories for the JSON file path")?;
    }

    let mut file = File::create(json_path).context(format!("Failed to create {json_path}"))?;
    file.write_all(json.as_bytes())
        .context(format!("Failed to write to {json_path}"))?;

    println!("Saved to {json_path}");
    Ok(())
}
