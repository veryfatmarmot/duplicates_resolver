use crate::types::{DuplicatesCollection, DuplicatesRegistry, FileDescr};
use chrono;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use walkdir::WalkDir;

// =============================================================================================

type FileCollection = HashMap<u64, HashSet<PathBuf>>;

// =============================================================================================

pub fn prescan_duplicates(root_path: &str) {
    println!(
        "[{}] Prescanning for duplicates in '{root_path}'",
        chrono::Local::now().format("%H:%M:%S")
    );

    type PrescanResult = HashMap<String, (usize, u64)>; // (count, total size)

    let collection = collect_files(root_path, 4); // Using 4 threads for prescan
    let total_files_count = collection.values().map(|v| v.len()).sum::<usize>();
    println!("Total files count:{total_files_count}");
    println!(
        "Estimated duplicates count: {} (in reality may be less)",
        collection.len()
    );

    let file_types: PrescanResult = collection 
        .into_iter()
        .flat_map(|(file_size, paths)|{
            paths.into_iter().map(move |path| (file_size, path))
        })
        .fold(PrescanResult::new(), |mut acc, (file_size, path)| {
            let ext: String = match path.extension()
            {
                Some(ext) => ext.to_string_lossy().to_string().to_lowercase(),
                None => "___".to_string(),
            };

            let entry = acc.entry(ext).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += file_size;
            acc
        });

    let mut file_types: Vec<(String, (usize, u64))> = file_types.into_iter().collect();
    file_types.sort_by(|a, b| b.1.1.cmp(&a.1.1)); // Sort by total size descending

    println!("File types distribution:");
    println!("Extension\tCount\t\tTotal Size");
    for (ext, (count, total_size)) in file_types {
        println!(
            "'{ext}'\t\t{count}\t\t{:.6} Gb",
            (total_size as f64) / ((1024 * 1024 * 1024) as f64)
        );
    }
}

pub fn collect_duplicates(root_path: &str, threads_count: u8) -> DuplicatesRegistry {
    println!(
        "[{}] Scanning (with {threads_count} threads) for duplicates in '{root_path}'",
        chrono::Local::now().format("%H:%M:%S")
    );

    let collection = collect_files(root_path, threads_count);
    let total_files_count = collection.values().map(|v| v.len()).sum::<usize>();
    println!("Total files count:{total_files_count}");

    let duplicates: DuplicatesCollection = extract_duplicates(collection, root_path, threads_count);
    println!("Duplicates count: {}", duplicates.len());

    let root_path = root_path.to_string().replace("\\", "/");
    DuplicatesRegistry {
        root_path,
        max_relative_path_len: get_max_relative_path_len(&duplicates),
        duplicates,
    }
}

fn create_thread_pool(threads_count: u8) -> rayon::ThreadPool {
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

    let pool = create_thread_pool(threads_count);
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
            .filter(|(file_size, _)| *file_size > 0)
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

fn extract_duplicates(
    collection: FileCollection,
    root_path: &str,
    threads_count: u8,
) -> DuplicatesCollection {
    let collection: FileCollection = collection
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();

    let total_files: usize = collection.values().map(|paths| paths.len()).sum();
    let progress_bar = ProgressBar::new(total_files as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} Computing hashes [{bar:40.cyan/blue}] {pos}/{len} ({eta_precise})")
            .expect("Failed to set progress style")
            .progress_chars("#>-"),
    );
    let progress_bar: Arc<Mutex<ProgressBar>> = Arc::new(Mutex::new(progress_bar));

    let pool = create_thread_pool(threads_count);
    let collection: DuplicatesCollection = pool.install(|| {
        collection
            .into_iter()
            .par_bridge()
            .map(|(_, paths)| {
                let file_count = paths.len();
                let result = extract_duplicate_descr(paths, root_path);
                match progress_bar.lock() {
                    Ok(ref pb) => pb.inc(file_count as u64),
                    Err(_) => panic!("Failed to lock progress bar"),
                }
                result
            })
            .reduce(DuplicatesCollection::new, |mut acc, mut element| {
                for (key, descrs) in &mut element {
                    acc.entry(key.clone())
                        .or_insert_with(Vec::new)
                        .append(descrs);
                }
                acc
            })
    });

    match progress_bar.lock() {
        Ok(ref pb) => pb.finish_with_message("All duplicates processed"),
        Err(_) => panic!("Failed to lock progress bar"),
    }

    collection
}

fn extract_duplicate_descr(paths: HashSet<PathBuf>, root_path: &str) -> DuplicatesCollection {
    let mut duplicates: Vec<(String, FileDescr)> = paths
        .into_iter()
        .map(|path| {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let folder_path = path
                .parent()
                .unwrap()
                .strip_prefix(root_path)
                .unwrap()
                .to_string_lossy()
                .to_string();
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
    for (hash, file_descr) in duplicates {
        result
            .entry(hash.clone())
            .or_insert_with(Vec::new)
            .push(file_descr);
    }

    result
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

fn get_max_relative_path_len(duplicates: &HashMap<String, Vec<FileDescr>>) -> u16 {
    let mut max_len: u16 = 0;

    for files in duplicates.values() {
        for descr in files {
            let path_len = (descr.folder_path.len() + descr.file_name.len() + 1) as u16; // +1 for the slash
            if path_len > max_len {
                max_len = path_len;
            }
        }
    }

    max_len
}
