use crate::types::{DuplicatesRegistry, FileDescr};
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

// ===================================================================================================

pub fn move_duplicates(
    registry: DuplicatesRegistry,
    target_root: &str,
    check_max_path_len: bool,
) -> Result<()> {
    let target_path = Path::new(target_root);
    let source_path = Path::new(&registry.root_path);

    if target_path.exists() {
        return Err(anyhow!("Target root already exists: {target_root}"));
    }

    if check_max_path_len && (target_root.len() + 1 + registry.max_relative_path_len as usize > 260)
    {
        return Err(anyhow!(
            "The resulting path length exceeds the maximum allowed length of 260 characters"
        ));
    }

    // Check if source and target paths overlap
    let target_canonical = target_path
        .canonicalize()
        .unwrap_or_else(|_| target_path.to_path_buf());
    let source_canonical = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());

    if target_canonical == source_canonical {
        return Err(anyhow!("Target path cannot be the same as source path"));
    }

    if target_canonical.starts_with(&source_canonical) {
        return Err(anyhow!("Target path cannot be inside source path"));
    }

    if source_canonical.starts_with(&target_canonical) {
        return Err(anyhow!("Source path cannot be inside target path"));
    }

    fs::create_dir_all(target_path).context("Failed to create target root directory")?;

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
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create target directories for '{dst_path:?}': {err}");
                    continue;
                }
            }

            if let Err(err) = fs::rename(&src_path, &dst_path) {
                eprintln!("Failed to move duplicate file '{src_path:?}' -> '{dst_path:?}': {err}");
                continue;
            }
        }
    }

    println!("All duplicates have been moved to: '{target_root}'");
    println!("Removing empty folders from source path: '{source_path:?}'");
    remove_empty_folders(source_path);

    Ok(())
}

fn src_path_from_descr(registry_root: &str, descr: &FileDescr) -> PathBuf {
    PathBuf::from(registry_root)
        .join(&descr.folder_path)
        .join(&descr.file_name)
}

fn remove_empty_folders(root_path: &Path) {
    let root = match fs::read_dir(root_path) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Failed to read directory: '{root_path:?}'");
            return;
        }
    };

    for entry in root {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to get directory entry: {e}");
                continue;
            }
        };

        let path = entry.path();
        if path.is_dir() {
            remove_empty_folders(&path);

            // After removing subfolders, check if the current folder is empty
            match fs::read_dir(&path) {
                Ok(mut dir) => {
                    if dir.next().is_none() {
                        if let Err(err) = fs::remove_dir(&path) {
                            eprintln!("Failed to remove empty directory '{path:?}': {err}");
                            continue;
                        };
                    }
                }
                Err(err) => {
                    eprintln!("Failed to read directory '{path:?}': {err}");
                    continue;
                }
            };
        }
    }
}
