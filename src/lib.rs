use rayon::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use walkdir::WalkDir;

// =============================================================================================

type FileCollection = HashMap<u64, HashSet<PathBuf>>;
type DuplicatesCollection = HashMap<String, Vec<FileDescr>>;

// =============================================================================================

#[derive(Debug, Serialize)]
struct FileDescr {
    folder_path: String,
    file_name: String,
}

// =============================================================================================

pub fn resolve_duplicates(root_path: &str, json_path: &str, threads_count: u8) {
    println!("Scanning (with {threads_count} threads) for duplicates in '{root_path}'");

    let collection = collect_files(root_path, threads_count);
    let total_files_count = collection.values().map(|v| v.len()).sum::<usize>();
    println!("Total files count:{total_files_count}");

    let collection: DuplicatesCollection = collect_duplicates(collection, threads_count);
    println!(
        "Duplicates count: {} - saved to {json_path}",
        collection.len()
    );

    save_to_json(collection, json_path);
}

fn create_tread_pool(threads_count: u8) -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads_count as usize)
        .build()
        .expect("Failed to build thread pool")
}

fn collect_files(root_path: &str, threads_count: u8) -> FileCollection {
    let extract_file_info = |entry: Result<walkdir::DirEntry, walkdir::Error>| -> (u64, PathBuf) {
        let entry = entry.expect("Failed to get WalkDir entry");
        let metadata = entry.metadata().expect("Failed to get metadata");
        let file_size = metadata.len();
        (file_size, entry.into_path())
    };

    let pool = create_tread_pool(threads_count);
    let collection: FileCollection = pool.install(|| {
        WalkDir::new(root_path)
            .follow_links(false)
            .into_iter()
            .filter(|e| match e {
                Ok(entry) => entry.file_type().is_file(),
                Err(err) => {
                    eprint!("WalkDir error: {err}");
                    false
                }
            })
            .par_bridge()
            .map(extract_file_info)
            .fold_with(FileCollection::new(), |mut acc, (key, value)| {
                acc.entry(key).or_insert_with(HashSet::new).insert(value);
                acc
            })
            .reduce(FileCollection::new, |mut acc, element| {
                for (key, values) in element {
                    acc.entry(key).or_insert_with(HashSet::new).extend(values);
                }
                acc
            })
    });

    collection
}

fn collect_duplicates(collection: FileCollection, threads_count: u8) -> DuplicatesCollection {
    let pool = create_tread_pool(threads_count);
    let collection: DuplicatesCollection = pool.install(|| {
        collection
            .into_iter()
            .filter(|(_, paths)| paths.len() > 1)
            .par_bridge()
            .map(|(_, paths)| extract_duplicate_descr(paths))
            .reduce(DuplicatesCollection::new, |mut acc, mut element| {
                for (key, descrs) in &mut element {
                    acc.entry(key.clone())
                        .or_insert_with(Vec::new)
                        .append(descrs);
                }
                acc
            })
    });

    collection
}

fn extract_duplicate_descr(paths: HashSet<PathBuf>) -> DuplicatesCollection {
    let mut duplicates: Vec<(String, FileDescr)> = paths
        .into_iter()
        .map(|path| {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let folder_path = path.parent().unwrap().to_string_lossy().to_string();
            let folder_path = folder_path.replace('\\', "/");
            let hash = calculate_hash(&path);
            (
                hash,
                FileDescr {
                    folder_path,
                    file_name,
                },
            )
        })
        .collect();

    duplicates.sort_by(|a, b| a.1.folder_path.cmp(&b.1.folder_path));

    let mut result = DuplicatesCollection::new();
    for (hash, file_descr) in duplicates.drain(..) {
        result
            .entry(hash.clone())
            .or_insert_with(Vec::new)
            .push(file_descr);
    }

    result
}

fn save_to_json(collection: DuplicatesCollection, save_path: &str) {
    let json = serde_json::to_string_pretty(&collection).expect("Failed to serialize to JSON");

    let path = PathBuf::from(save_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create directories");
    }

    let mut file = File::create(save_path).expect("Failed to create {save_path}");
    file.write_all(json.as_bytes())
        .expect("Failed to write to {save_path}");
}

fn calculate_hash(path: &PathBuf) -> String {
    let mut file = File::open(path).expect("Failed to open file for hashing");
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .expect("Failed to read file for hashing");
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash_result = hasher.finalize();
    format!("{:x}", hash_result)
}
