//! Source-level refactoring operations for newc projects.
//!
//! Currently provides function renaming (project-wide text substitution) and
//! function moving (across modules with automatic header re-sync).

use std::path::Path;

use crate::analysis::remove_function_from_source_pub;
use crate::error::Result;
use crate::sync::{extract_function_implementations, sync_module};

/// Replace every occurrence of `old_name(` with `new_name(` in all `.c` and `.h`
/// files under the project.
///
/// # Returns
/// The number of files that were modified.
///
/// # Errors
/// Returns an IO error if any file cannot be read or written.
pub fn rename_function(root: &Path, old_name: &str, new_name: &str) -> Result<usize> {
    let pattern = format!("{old_name}(");
    let replacement = format!("{new_name}(");
    let mut count = 0;

    for dir in [root.join("src"), root.join("include")] {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "c" && ext != "h" { continue; }
            let content = std::fs::read_to_string(&path)?;
            if content.contains(&pattern) {
                std::fs::write(&path, content.replace(&pattern, &replacement))?;
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Move function `fname` from `from_module` to `to_module` within the same project.
///
/// Removes the function (comment + signature + body) from its current source file,
/// appends it to the target module's source file, and re-syncs both headers.
///
/// # Errors
/// Returns an error if `fname` is not found in `from_module`, or on any IO failure.
pub fn move_function(root: &Path, from_module: &str, to_module: &str, fname: &str) -> Result<()> {
    let src_path = root.join("src").join(format!("{from_module}.c"));
    let tgt_path = root.join("src").join(format!("{to_module}.c"));

    // Read source and locate the function
    let src_content = std::fs::read_to_string(&src_path)?;
    let funcs = extract_function_implementations(&src_content);
    let func = funcs.iter().find(|f| f.name == fname).ok_or_else(|| {
        crate::error::NewcError::Other(format!("function '{fname}' not found in {from_module}"))
    })?;

    // Reconstruct full function text: optional comment + signature + body
    let mut func_text = String::new();
    if !func.comment.is_empty() {
        func_text.push_str(&func.comment);
        func_text.push('\n');
    }
    func_text.push_str(&func.signature);
    func_text.push('\n');
    func_text.push_str(&func.body);
    func_text.push('\n');

    // Append to target module
    let mut tgt = std::fs::read_to_string(&tgt_path)?;
    if !tgt.ends_with('\n') { tgt.push('\n'); }
    tgt.push('\n');
    tgt.push_str(&func_text);
    std::fs::write(&tgt_path, tgt)?;

    // Remove from source module
    remove_function_from_source_pub(&src_path, fname)?;

    // Re-sync both headers
    sync_module(root, from_module)?;
    sync_module(root, to_module)?;

    Ok(())
}
