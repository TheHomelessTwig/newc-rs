//! Source-level refactoring operations for newc projects.
//!
//! Currently provides function renaming (project-wide text substitution) and
//! function moving (across modules with automatic header re-sync).

use std::path::Path;

use crate::analysis::remove_function_from_source_pub;
use crate::error::Result;
use crate::sync::{extract_function_implementations, sync_module};

/// Replace every call/definition/prototype of `old_name` with `new_name` in all
/// `.c` and `.h` files under the project. Matches at word boundaries, so a
/// function whose name merely contains `old_name` (e.g. `do_old_name`) is
/// left alone.
///
/// # Returns
/// The number of files that were modified.
///
/// # Errors
/// Returns an IO error if any file cannot be read or written.
pub fn rename_function(root: &Path, old_name: &str, new_name: &str) -> Result<usize> {
    let re = regex::Regex::new(&format!(r"\b{}\s*\(", regex::escape(old_name)))
        .expect("escaped identifier is a valid regex");
    let replacement = format!("{new_name}(");
    let mut count = 0;

    for dir in [root.join("src"), root.join("include")] {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "c" && ext != "h" { continue; }
            let content = std::fs::read_to_string(&path)?;
            if re.is_match(&content) {
                std::fs::write(&path, re.replace_all(&content, replacement.as_str()).as_ref())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().to_path_buf();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("include")).unwrap();
        fs::write(
            root.join("src/util.c"),
            "int foo(void)\n{\n    return 1;\n}\n\nint do_foo(void)\n{\n    return foo();\n}\n",
        )
        .unwrap();
        fs::write(root.join("include/util.h"), "int foo(void);\n\nint do_foo(void);\n").unwrap();
        fs::write(root.join("src/other.c"), "int bar(void)\n{\n    return 2;\n}\n").unwrap();
        fs::write(root.join("include/other.h"), "int bar(void);\n").unwrap();
        (tmp, root)
    }

    #[test]
    fn rename_leaves_similar_names_alone() {
        let (_tmp, root) = fixture();
        rename_function(&root, "foo", "renamed").unwrap();
        let src = fs::read_to_string(root.join("src/util.c")).unwrap();
        assert!(src.contains("int renamed(void)"));
        assert!(src.contains("return renamed();"));
        // do_foo must be untouched — its name only contains "foo"
        assert!(src.contains("int do_foo(void)"));
        let hdr = fs::read_to_string(root.join("include/util.h")).unwrap();
        assert!(hdr.contains("renamed(void);"));
        assert!(hdr.contains("do_foo(void);"));
    }

    #[test]
    fn move_function_transfers_and_resyncs_headers() {
        let (_tmp, root) = fixture();
        move_function(&root, "util", "other", "foo").unwrap();
        let src = fs::read_to_string(root.join("src/util.c")).unwrap();
        assert!(!src.contains("int foo(void)"));
        let tgt = fs::read_to_string(root.join("src/other.c")).unwrap();
        assert!(tgt.contains("int foo(void)"));
        let tgt_hdr = fs::read_to_string(root.join("include/other.h")).unwrap();
        assert!(tgt_hdr.contains("foo(void);"));
        let src_hdr = fs::read_to_string(root.join("include/util.h")).unwrap();
        assert!(!src_hdr.contains("int foo(void);"));
    }

    #[test]
    fn move_missing_function_errors() {
        let (_tmp, root) = fixture();
        assert!(move_function(&root, "util", "other", "nope").is_err());
    }
}
