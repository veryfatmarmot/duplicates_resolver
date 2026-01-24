use crate::types::{DuplicatesRegistry, FileDescr};
use std::fs;
use std::path::{Path, PathBuf};

// ===================================================================================================

pub fn move_duplicates(registry: DuplicatesRegistry, target_root: &str) {
    let target_path = Path::new(target_root);
    let source_path = Path::new(&registry.root_path);

    if target_path.exists() {
        panic!("Target root already exists: {target_root}");
    }

    // Check if source and target paths overlap
    let target_canonical = target_path.canonicalize().unwrap_or_else(|_| target_path.to_path_buf());
    let source_canonical = source_path.canonicalize().unwrap_or_else(|_| source_path.to_path_buf());
    
    if target_canonical == source_canonical {
        panic!("Target path cannot be the same as source path");
    }
    
    if target_canonical.starts_with(&source_canonical) {
        panic!("Target path cannot be inside source path");
    }
    
    if source_canonical.starts_with(&target_canonical) {
        panic!("Source path cannot be inside target path");
    }

    fs::create_dir_all(target_path).expect("Failed to create target root directory");

    for (hash, files) in registry.duplicates {
        if files.len() < 2 {
            eprintln!("the block '{hash}' has less than 2 files, skipping");
            continue; // No duplicates to move
        }

        let mut files_iter = files.iter();
        let original_path = src_path_from_descr(&registry.root_path, files_iter.next().unwrap());
        if !original_path.exists() {
            eprintln!("Original file does not exist, skipping the block. '{original_path:?}'");
            continue;
        }

        for duplicate in files_iter {
            let src_path = src_path_from_descr(&registry.root_path, duplicate);
            if !src_path.exists() {
                eprintln!("Source file does not exist, skipping: {src_path:?}");
                continue;
            }

            let dst_path = PathBuf::from(target_root)
                .join(&duplicate.folder_path)
                .join(&duplicate.file_name);

            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create target directories");
            }

            fs::rename(&src_path, &dst_path).expect(&format!(
                "Failed to move duplicate file '{src_path:?}' -> '{dst_path:?}'"
            ));
        }
    }
}

fn src_path_from_descr(registry_root: &str, descr: &FileDescr) -> PathBuf {
    PathBuf::from(registry_root)
        .join(&descr.folder_path)
        .join(&descr.file_name)
}
