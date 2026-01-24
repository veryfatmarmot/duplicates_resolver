use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDescr {
    pub folder_path: String,
    pub file_name: String,
}

pub type DuplicatesCollection = HashMap<String, Vec<FileDescr>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicatesRegistry {
    pub root_path: String,
    pub max_relative_path_len: u16,
    pub duplicates: DuplicatesCollection,
}
