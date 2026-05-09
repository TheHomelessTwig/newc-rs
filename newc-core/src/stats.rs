use std::path::Path;

use crate::sync::extract_signatures;

#[derive(Debug, Clone, Default)]
pub struct ProjectStats {
    pub total_functions: usize,
    pub total_loc: usize,
    pub total_source_lines: usize,
    pub module_stats: Vec<ModuleStats>,
}

#[derive(Debug, Clone)]
pub struct ModuleStats {
    pub name: String,
    pub functions: usize,
    pub loc: usize,
    pub source_lines: usize,
}

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

    module_stats.sort_by(|a, b| b.functions.cmp(&a.functions));

    ProjectStats { total_functions, total_loc, total_source_lines, module_stats }
}

/// Non-blank, non-comment-only lines.
fn count_loc(content: &str) -> usize {
    let mut in_block_comment = false;
    let mut count = 0;
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if in_block_comment {
            if t.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if t.starts_with("/*") {
            if !t.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        if t.starts_with("//") || t.starts_with("*") {
            continue;
        }
        count += 1;
    }
    count
}
