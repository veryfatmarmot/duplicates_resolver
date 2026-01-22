use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use walkdir::WalkDir;

// =============================================================================================

type FileCollection = HashMap<u64, HashSet<PathBuf>>;
type DuplicatesCollection = HashMap<String, Vec<FileDescr>>;

// =============================================================================================

#[derive(Debug)]
struct FileDescr {
    relative_folder_path: String,
    file_name: String,
}

// =============================================================================================

pub fn resolve_duplicates(root_path: &str, threads_count: u8) {
    println!("Scanning (with {threads_count} threads) for duplicates in '{root_path}'");

    let collection = collect_files(root_path, threads_count);

    // leave duplicates only
    let collection: DuplicatesCollection = collection
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(_size, paths)| {
            let mut duplicates: Vec<FileDescr> = paths.into_iter()
            .map(|path| {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let relative_folder_path = path.parent().unwrap().strip_prefix(root_path)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                FileDescr {relative_folder_path, file_name}
            }).collect();

            duplicates.sort_by(|a, b| a.relative_folder_path.cmp(&b.relative_folder_path));

            (duplicates[0].file_name.clone(), duplicates)
        })
        .collect();    

    println!("Collection:\n{:#?}", collection);
}

fn collect_files(root_path: &str, threads_count: u8) -> FileCollection {    
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads_count as usize)
        .build()
        .expect("Failed to build thread pool");

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
            .map(|entry| {
                let entry = entry.expect("Failed to get WalkDir entry");
                let metadata = entry.metadata().expect("Failed to get metadata");
                let file_size = metadata.len();               
                (file_size, entry.into_path())
            })
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