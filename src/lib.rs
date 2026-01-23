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
    relative_folder_path: String,
    file_name: String,
    hash: String,
}

// =============================================================================================

pub fn resolve_duplicates(root_path: &str, json_path: &str, threads_count: u8) {
    println!("Scanning (with {threads_count} threads) for duplicates in '{root_path}'");

    let collection = collect_files(root_path, threads_count);
    let total_files_count = collection.values().map(|v| v.len()).sum::<usize>();

    let collection: DuplicatesCollection = collect_duplicates(collection, root_path, threads_count);
    let duplicates_count = collection.len();

    save_to_json(collection, json_path);

    println!("Total files count:{total_files_count}");
    println!("Duplicates count: {duplicates_count} - saved to {json_path}");
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

fn collect_duplicates(
    collection: FileCollection,
    root_path: &str,
    threads_count: u8,
) -> DuplicatesCollection {
    let pool = create_tread_pool(threads_count);
    let collection: DuplicatesCollection = pool.install(|| {
        collection
            .into_iter()
            .filter(|(_, paths)| paths.len() > 1)
            .par_bridge()
            .map(|(_, paths)| extract_duplicate_descr(paths, root_path))
            .collect()
    });

    collection
}

fn extract_duplicate_descr(paths: HashSet<PathBuf>, root_path: &str) -> (String, Vec<FileDescr>) {
    let mut duplicates: Vec<FileDescr> = paths
        .into_iter()
        .map(|path| {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let relative_folder_path = path
                .parent()
                .unwrap()
                .strip_prefix(root_path)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let relative_folder_path = relative_folder_path.replace('\\', "/");
            let hash = calculate_hash(&path);
            FileDescr {
                relative_folder_path,
                file_name,
                hash,
            }
        })
        .collect();

    duplicates.sort_by(|a, b| a.relative_folder_path.cmp(&b.relative_folder_path));

    (duplicates[0].file_name.clone(), duplicates)
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
