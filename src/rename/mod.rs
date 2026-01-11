// Rename strategies: find/replace and iteration numbering

use crate::theme::MAX_PATTERN_LENGTH;
use crate::types::{FileEntry, RenamePreview};
use anyhow::Result;
use regex::RegexBuilder;
use std::collections::HashMap;
use std::sync::Arc;

// Applies find/replace pattern to filenames
pub fn apply_find_replace(
    files: &[FileEntry],
    pattern: &str,
    replacement: &str,
    use_regex: bool,
    case_sensitive: bool,
) -> Result<Vec<RenamePreview>> {
    if pattern.is_empty() {
        return Ok(Vec::new());
    }

    if pattern.len() > MAX_PATTERN_LENGTH {
        anyhow::bail!("Pattern too long (max {} chars)", MAX_PATTERN_LENGTH);
    }

    let mut previews = Vec::new();

    if use_regex {
        let regex = RegexBuilder::new(pattern)
            .case_insensitive(!case_sensitive)
            .size_limit(1024 * 1024)
            .build()
            .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

        for file in files {
            let new_name = regex.replace_all(&file.name, replacement).to_string();
            if new_name != file.name.as_str() {
                previews.push(RenamePreview {
                    original_path: file.path.clone(),
                    original_name: Arc::clone(&file.name),
                    new_name,
                    has_conflict: false,
                });
            }
        }
    } else {
        for file in files {
            let new_name = if case_sensitive {
                file.name.replace(pattern, replacement)
            } else {
                replace_case_insensitive(&file.name, pattern, replacement)
            };
            if new_name != file.name.as_str() {
                previews.push(RenamePreview {
                    original_path: file.path.clone(),
                    original_name: Arc::clone(&file.name),
                    new_name,
                    has_conflict: false,
                });
            }
        }
    }

    detect_conflicts(&mut previews);
    Ok(previews)
}

// Applies sequential numbering using template with {n} placeholder
pub fn apply_iteration_numbering(
    files: &[FileEntry],
    template: &str,
    start_number: u32,
    padding: usize,
) -> Result<Vec<RenamePreview>> {
    if !template.contains("{n}") {
        anyhow::bail!("Template must contain {{n}} placeholder");
    }

    let mut previews = Vec::new();

    for (index, file) in files.iter().enumerate() {
        let number = start_number.saturating_add(index as u32);
        let formatted_number = format!("{:0>width$}", number, width = padding);
        let extension = file
            .path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let new_name = format!(
            "{}{}",
            template.replace("{n}", &formatted_number),
            extension
        );

        previews.push(RenamePreview {
            original_path: file.path.clone(),
            original_name: Arc::clone(&file.name),
            new_name,
            has_conflict: false,
        });
    }

    detect_conflicts(&mut previews);
    Ok(previews)
}

// Case-insensitive string replacement
fn replace_case_insensitive(text: &str, pattern: &str, replacement: &str) -> String {
    let regex = RegexBuilder::new(&regex::escape(pattern))
        .case_insensitive(true)
        .build()
        .expect("Escaped pattern should be valid");
    regex.replace_all(text, replacement).to_string()
}

// Marks duplicate target names as conflicts
fn detect_conflicts(previews: &mut [RenamePreview]) {
    let mut counts: HashMap<String, usize> = HashMap::with_capacity(previews.len());
    for preview in previews.iter() {
        *counts.entry(preview.new_name.to_lowercase()).or_insert(0) += 1;
    }
    for preview in previews.iter_mut() {
        if counts
            .get(&preview.new_name.to_lowercase())
            .copied()
            .unwrap_or(0)
            > 1
        {
            preview.has_conflict = true;
        }
    }
}
