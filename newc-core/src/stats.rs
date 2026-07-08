//! Source-code statistics for newc projects.
//!
//! Counts functions, lines of code (LOC), and raw source lines per module.
//! LOC excludes blank lines and comment-only lines.

use std::path::Path;

use crate::sync::extract_signatures;

/// Aggregate statistics for an entire project.
#[derive(Debug, Clone, Default)]
pub struct ProjectStats {
    /// Total number of functions across all modules (excludes `main()`).
    pub total_functions: usize,
    /// Total LOC across all source files including `main.c`.
    pub total_loc: usize,
    /// Total raw source lines including `main.c`.
    pub total_source_lines: usize,
    /// Per-module breakdown, sorted descending by function count.
    pub module_stats: Vec<ModuleStats>,
}

/// Statistics for a single module (one `.c` file, excluding `main.c`).
#[derive(Debug, Clone)]
pub struct ModuleStats {
    /// Module name (stem of the `.c` filename).
    pub name: String,
    /// Number of functions defined in the module.
    pub functions: usize,
    /// Lines of code (non-blank, non-comment lines).
    pub loc: usize,
    /// Total raw line count including comments and blank lines.
    pub source_lines: usize,
}

/// Compute statistics for the project at `root`.
///
/// Returns a zeroed [`ProjectStats`] if `src/` cannot be read.
pub fn compute(root: &Path) -> ProjectStats {
    let src_dir = root.join("src");
    let Ok(entries) = std::fs::read_dir(&src_dir) else {
        return ProjectStats::default();
    };

    let mut module_stats: Vec<ModuleStats> = Vec::new();
    let mut main_loc = 0;
    let mut main_source = 0;

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("c"))
        .map(|e| e.path())
        .collect();
    paths.sort();

    for path in &paths {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let source_lines = content.lines().count();
        let loc = count_loc(&content);
        let functions = if name == "main" {
            main_loc = loc;
            main_source = source_lines;
            0
        } else {
            extract_signatures(&content).len()
        };
        if name != "main" {
            module_stats.push(ModuleStats { name, functions, loc, source_lines });
        }
    }

    let total_functions = module_stats.iter().map(|m| m.functions).sum();
    let total_loc = module_stats.iter().map(|m| m.loc).sum::<usize>() + main_loc;
    let total_source_lines = module_stats.iter().map(|m| m.source_lines).sum::<usize>() + main_source;

    module_stats.sort_by_key(|m| std::cmp::Reverse(m.functions));

    ProjectStats { total_functions, total_loc, total_source_lines, module_stats }
}

/// Non-blank, non-comment-only lines.
fn count_loc(content: &str) -> usize {
    let mut in_block_comment = false;
    let mut count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        count += 1;
    }
    count
}
