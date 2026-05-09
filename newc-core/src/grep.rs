use std::path::Path;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: String,
    pub line_no: usize,
    pub text: String,
    /// Module name if the file is a known .c source file (not main.c)
    pub module: Option<String>,
}

pub fn search(root: &Path, query: &str) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }
    let q = query.to_lowercase();
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
                if line.to_lowercase().contains(&q) {
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
