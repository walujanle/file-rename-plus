// File operations: directory scanning and atomic renaming

use crate::types::{FileEntry, RenamePreview};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Scans directory and returns files sorted naturally (like File Explorer)
pub fn scan_directory(path: &str) -> Result<Vec<FileEntry>> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    if !path.is_dir() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        return Ok(vec![FileEntry {
            path: path.to_path_buf(),
            name: Arc::new(name),
        }]);
    }

    let entries =
        fs::read_dir(path).with_context(|| format!("Failed to read: {}", path.display()))?;
    let mut files = Vec::new();

    for entry in entries {
        let entry = entry.with_context(|| "Failed to read entry")?;
        let file_path = entry.path();

        if file_path.is_dir() {
            continue;
        }

        let name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        files.push(FileEntry {
            path: file_path,
            name: Arc::new(name),
        });
    }

    files.sort_by(|a, b| natural_cmp(&a.name, &b.name));
    Ok(files)
}

// Natural sort: compares numbers numerically within strings
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let a_num = extract_number(&mut a_chars);
                    let b_num = extract_number(&mut b_chars);
                    match a_num.cmp(&b_num) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                }
                let ac_lower = ac.to_ascii_lowercase();
                let bc_lower = bc.to_ascii_lowercase();
                match ac_lower.cmp(&bc_lower) {
                    std::cmp::Ordering::Equal => {
                        a_chars.next();
                        b_chars.next();
                    }
                    other => return other,
                }
            }
        }
    }
}

// Extracts consecutive digits as a number
fn extract_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> u64 {
    let mut num: u64 = 0;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num = num
                .saturating_mul(10)
                .saturating_add(u64::from(c.to_digit(10).unwrap_or(0)));
            chars.next();
        } else {
            break;
        }
    }
    num
}

// Executes renames atomically using two-phase temporary rename
pub fn validate_and_rename(previews: &[RenamePreview]) -> Result<usize> {
    if previews.is_empty() {
        return Ok(0);
    }

    let mut target_names: HashSet<PathBuf> = HashSet::new();
    let original_paths: HashSet<PathBuf> =
        previews.iter().map(|p| p.original_path.clone()).collect();

    for preview in previews {
        let target_path = preview
            .original_path
            .parent()
            .unwrap_or(&preview.original_path)
            .join(&preview.new_name);

        if target_path.exists() && !original_paths.contains(&target_path) {
            anyhow::bail!("Target exists: {}", target_path.display());
        }
        if target_names.contains(&target_path) {
            anyhow::bail!("Duplicate target: {}", preview.new_name);
        }
        target_names.insert(target_path);
    }

    let temp_prefix = format!(".rename_temp_{}_", std::process::id());
    let mut temp_renames: Vec<(PathBuf, PathBuf)> = Vec::new();

    for preview in previews {
        if preview.original_name.as_str() == preview.new_name {
            continue;
        }
        let parent = preview
            .original_path
            .parent()
            .unwrap_or(&preview.original_path);
        let temp_path = parent.join(format!("{}{}", temp_prefix, preview.new_name));
        let final_path = parent.join(&preview.new_name);

        fs::rename(&preview.original_path, &temp_path)
            .with_context(|| format!("Failed to rename: {}", preview.original_path.display()))?;
        temp_renames.push((temp_path, final_path));
    }

    let mut renamed_count = 0;
    for (temp_path, final_path) in temp_renames {
        fs::rename(&temp_path, &final_path)
            .with_context(|| format!("Failed to finalize: {}", final_path.display()))?;
        renamed_count += 1;
    }

    Ok(renamed_count)
}
