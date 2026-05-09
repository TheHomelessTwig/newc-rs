use std::path::Path;

use crate::{build_history, meta, notes, stats};

pub fn generate(root: &Path, project_name: &str) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!("# {project_name}\n\n"));

    // Metadata
    let meta = meta::load(root);
    if !meta.is_empty() {
        md.push_str("## Project Info\n\n");
        if !meta.course.is_empty() {
            md.push_str(&format!("- **Course:** {}\n", meta.course));
        }
        if !meta.assignment.is_empty() {
            md.push_str(&format!("- **Assignment:** {}\n", meta.assignment));
        }
        if !meta.due_date.is_empty() {
            md.push_str(&format!("- **Due:** {}\n", meta.due_date));
        }
        if let Some(marks) = meta.marks_display() {
            md.push_str(&format!("- **Marks:** {marks}\n"));
        }
        if !meta.notes.is_empty() {
            md.push_str(&format!("\n{}\n", meta.notes));
        }
        md.push('\n');
    }

    // Stats
    let s = stats::compute(root);
    md.push_str("## Statistics\n\n");
    md.push_str("| Module | Functions | LOC |\n");
    md.push_str("|---|---|---|\n");
    for m in &s.module_stats {
        md.push_str(&format!("| {} | {} | {} |\n", m.name, m.functions, m.loc));
    }
    md.push_str(&format!("| **Total** | **{}** | **{}** |\n\n", s.total_functions, s.total_loc));

    // Modules + function list
    md.push_str("## Modules\n\n");
    let src_dir = root.join("src");
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|x| x.to_str()) == Some("c")
                    && e.file_name() != "main.c"
            })
            .map(|e| e.path())
            .collect();
        paths.sort();
        for path in &paths {
            let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            md.push_str(&format!("### {name}\n\n"));
            if let Ok(content) = std::fs::read_to_string(path) {
                let fns = crate::sync::extract_function_implementations(&content);
                for f in &fns {
                    md.push_str(&format!("- `{}`\n", f.signature));
                }
            }
            md.push('\n');
        }
    }

    // Build history
    let records = build_history::load(root);
    if !records.is_empty() {
        md.push_str("## Build History (last 10)\n\n");
        md.push_str("| Timestamp | Target | Result | Duration |\n");
        md.push_str("|---|---|---|---|\n");
        for rec in records.iter().rev().take(10) {
            let result = if rec.succeeded() { "✓ OK" } else { "✗ Failed" };
            md.push_str(&format!("| {} | {} | {} | {} |\n",
                rec.timestamp, rec.target, result, rec.duration_str()));
        }
        md.push('\n');
    }

    // Notes
    let project_notes = notes::load(root);
    if !project_notes.is_empty() {
        md.push_str("## Notes\n\n");
        md.push_str(&project_notes);
        md.push('\n');
    }

    md
}
