//! Regex-based text search across a project's C source and header files.

use std::path::Path;
use regex::RegexBuilder;

fn build_pattern(query: &str) -> regex::Regex {
    RegexBuilder::new(query)
        .case_insensitive(true)
        .build()
        .unwrap_or_else(|_| {
            RegexBuilder::new(&regex::escape(query))
                .case_insensitive(true)
                .build()
                .expect("escaped literal always valid")
        })
}

/// A single line that would be (or was) changed by a project-wide replace.
#[derive(Debug, Clone)]
pub struct ReplacePreview {
    pub file: String,
    pub line_no: usize,
    pub before: String,
    pub after: String,
}

/// Preview a project-wide find/replace without writing any files.
///
/// Same regex-or-literal fallback as [`search`]. Returns one entry per
/// matching line, showing the line before and after substitution.
pub fn preview_replacements(root: &Path, pattern: &str, replacement: &str) -> Vec<ReplacePreview> {
    if pattern.is_empty() {
        return Vec::new();
    }
    let re = build_pattern(pattern);
    let mut previews = Vec::new();

    for dir in [root.join("src"), root.join("include")] {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| matches!(p.extension().and_then(|x| x.to_str()), Some("c") | Some("h")))
            .collect();
        paths.sort();

        for path in &paths {
            let Ok(content) = std::fs::read_to_string(path) else { continue };
            let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    previews.push(ReplacePreview {
                        file: file_name.clone(),
                        line_no: i + 1,
                        before: line.to_string(),
                        after: re.replace_all(line, replacement).into_owned(),
                    });
                }
            }
        }
    }
    previews
}

/// Apply a project-wide find/replace to every matching `.c`/`.h` file under the project.
///
/// # Returns
/// The number of files modified.
///
/// # Errors
/// Returns an IO error if any matching file cannot be read or written.
pub fn apply_replacements(root: &Path, pattern: &str, replacement: &str) -> std::io::Result<usize> {
    if pattern.is_empty() {
        return Ok(0);
    }
    let re = build_pattern(pattern);
    let mut modified = 0;

    for dir in [root.join("src"), root.join("include")] {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| matches!(p.extension().and_then(|x| x.to_str()), Some("c") | Some("h")))
            .collect();
        paths.sort();

        for path in &paths {
            let content = std::fs::read_to_string(path)?;
            if re.is_match(&content) {
                let updated = re.replace_all(&content, replacement);
                std::fs::write(path, updated.as_bytes())?;
                modified += 1;
            }
        }
    }
    Ok(modified)
}

/// A single line that matched a search query.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Filename (basename only) of the matching file.
    pub file: String,
    /// 1-based line number of the match.
    pub line_no: usize,
    /// Trimmed text of the matching line.
    pub text: String,
    /// Module name if the file is a known `.c` source file (not `main.c`).
    pub module: Option<String>,
}

/// Search `src/` and `include/` for `query`.
///
/// If `query` is a valid regex it is used as-is (case-insensitive); invalid
/// regex is automatically escaped and treated as a literal substring.
/// Returns an empty vec for an empty query.
pub fn search(root: &Path, query: &str) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }

    let pattern = build_pattern(query);

    let mut results = Vec::new();

    let search_dirs = [root.join("src"), root.join("include")];
    for dir in &search_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
                ext == "c" || ext == "h"
            })
            .collect();
        paths.sort();

        for path in &paths {
            let Ok(content) = std::fs::read_to_string(path) else { continue };
            let file_name = path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let module = if path.extension().and_then(|x| x.to_str()) == Some("c")
                && file_name != "main.c"
            {
                path.file_stem().map(|s| s.to_string_lossy().into_owned())
            } else {
                None
            };

            for (i, line) in content.lines().enumerate() {
                if pattern.is_match(line) {
                    results.push(SearchResult {
                        file: file_name.clone(),
                        line_no: i + 1,
                        text: line.trim_end().to_string(),
                        module: module.clone(),
                    });
                }
            }
        }
    }

    results
}
