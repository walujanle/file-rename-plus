// Shared types used across modules

use std::path::PathBuf;
use std::sync::Arc;

/// Represents a file entry in the list with shared name
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: Arc<String>,
}

/// Represents a preview of a rename operation
#[derive(Debug, Clone)]
pub struct RenamePreview {
    pub original_path: PathBuf,
    pub original_name: Arc<String>,
    pub new_name: String,
    pub has_conflict: bool,
}

/// Application operating modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    FindReplace,
    Iteration,
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::FindReplace => write!(f, "Find & Replace"),
            AppMode::Iteration => write!(f, "Iteration Numbering"),
        }
    }
}
