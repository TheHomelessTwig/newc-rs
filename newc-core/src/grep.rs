//! Regex-based text search across a project's C source and header files.

use std::path::Path;
use regex::RegexBuilder;

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

    // Build regex: try query as regex, fall back to literal escape.
    let pattern = RegexBuilder::new(query)
        .case_insensitive(true)
        .build()
        .unwrap_or_else(|_| {
            RegexBuilder::new(&regex::escape(query))
                .case_insensitive(true)
                .build()
                .expect("escaped literal always valid")
        });

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
