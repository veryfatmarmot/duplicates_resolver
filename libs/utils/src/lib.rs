mod scope_time_logger;
pub use crate::scope_time_logger::ScopeTimeLogger;

pub mod thread_pool;

// =======================================================================================================

use std::{
    env,
    path::{Path, PathBuf},
};

// =======================================================================================================

/// converts a path to an absolute path
pub fn path_to_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap().join(path)
    }
}

/// produces a proper path from the project root
pub fn path_from_root(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(relative_path)
}