//! Module management — listing, adding, and removing `.c`/`.h` module pairs.
//!
//! A "module" in newc is a pair of files: `src/<name>.c` and `include/<name>.h`.
//! Adding a module also injects the appropriate `#include` into `main.c`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{NewcError, Result};

/// Metadata about a single newc module.
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name (stem of the `.h`/`.c` filenames).
    pub name: String,
    /// Absolute path to `include/<name>.h`.
    pub header: PathBuf,
    /// Absolute path to `src/<name>.c`.
    pub source: PathBuf,
    /// Number of functions defined in `source`.
    pub function_count: usize,
}

/// List all modules in the project, sorted by name.
///
/// A module is any `.h` file in `include/` that has a matching entry in `src/`.
///
/// # Errors
/// Returns an IO error if `include/` cannot be read.
pub fn list_modules(root: &Path) -> Result<Vec<Module>> {
    let include_dir = root.join("include");
    let mut modules = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(&include_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("h")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let header = entry.path();
        let name = header
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let source = root.join("src").join(format!("{name}.c"));
        let function_count = count_functions_in_source(&source);
        modules.push(Module { name, header, source, function_count });
    }
    Ok(modules)
}

/// Returns true if `s` is a valid C identifier: `[a-zA-Z_][a-zA-Z0-9_]*`, max 63 chars.
pub fn is_valid_c_ident(s: &str) -> bool {
    if s.is_empty() || s.len() > 63 {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Create a new module (`.c` + `.h` pair) and inject its `#include` into `main.c`.
///
/// # Errors
/// Returns [`NewcError::InvalidName`] if `name` is not a valid C identifier,
/// [`NewcError::ModuleExists`] if the files already exist, or an IO error.
pub fn add_module(root: &Path, name: &str) -> Result<()> {
    if !is_valid_c_ident(name) {
        return Err(NewcError::InvalidName(name.to_string()));
    }
    let header = root.join("include").join(format!("{name}.h"));
    let source = root.join("src").join(format!("{name}.c"));

    if header.exists() || source.exists() {
        return Err(NewcError::ModuleExists(name.to_string()));
    }

    let spdx = crate::config::ProjectConfig::load_from(root)
        .and_then(|c| c.license)
        .map(|id| crate::license::spdx_line(&id))
        .unwrap_or_default();

    let guard = name.to_uppercase() + "_H";
    let header_content = format!(
        "{spdx}#ifndef {guard}\n#define {guard}\n\n/* SYNC_IGNORE_START */\n\n/* SYNC_IGNORE_END */\n\n#endif\n"
    );
    let source_content = format!(
        "{spdx}#include <stdio.h>\n#include \"{name}.h\"\n\n/*\n * Function:\n *     ...\n * Input:\n *     ...\n * Output:\n *     ...\n * Algorithm:\n *     ...\n */\n\n"
    );

    fs::write(&header, header_content)?;
    fs::write(&source, source_content)?;

    // Inject #include after last #include in main.c
    let main_c = root.join("src").join("main.c");
    if main_c.exists() {
        inject_include(&main_c, name)?;
    }

    Ok(())
}

/// Delete a module's `.c` and `.h` files and remove its `#include` from all `.c` files.
///
/// # Errors
/// Returns [`NewcError::ModuleNotFound`] if neither file exists, or an IO error.
pub fn remove_module(root: &Path, name: &str) -> Result<()> {
    let header = root.join("include").join(format!("{name}.h"));
    let source = root.join("src").join(format!("{name}.c"));

    if !header.exists() && !source.exists() {
        return Err(NewcError::ModuleNotFound(name.to_string()));
    }

    if header.exists() {
        fs::remove_file(&header)?;
    }
    if source.exists() {
        fs::remove_file(&source)?;
    }

    // Remove #include "name.h" from all .c files
    let src_dir = root.join("src");
    if let Ok(entries) = fs::read_dir(&src_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) == Some("c") {
                remove_include_line(&path, name)?;
            }
        }
    }

    Ok(())
}

fn inject_include(main_c: &Path, module_name: &str) -> Result<()> {
    let content = fs::read_to_string(main_c)?;
    let include_line = format!("#include \"{module_name}.h\"");

    // Don't double-inject
    if content.contains(&include_line) {
        return Ok(());
    }

    let lines: Vec<&str> = content.lines().collect();
    let last_include = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with("#include"))
        .map(|(i, _)| i)
        .next_back();

    let new_content = if let Some(idx) = last_include {
        let mut out = lines[..=idx].join("\n");
        out.push('\n');
        out.push_str(&include_line);
        out.push('\n');
        if idx + 1 < lines.len() {
            out.push_str(&lines[idx + 1..].join("\n"));
            out.push('\n');
        }
        out
    } else {
        format!("{include_line}\n{content}")
    };

    fs::write(main_c, new_content)?;
    Ok(())
}

fn remove_include_line(path: &Path, module_name: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let target = format!("#include \"{module_name}.h\"");
    let new_content: String = content
        .lines()
        .filter(|l| l.trim() != target.trim())
        .collect::<Vec<_>>()
        .join("\n");
    // preserve trailing newline
    let new_content = if content.ends_with('\n') {
        new_content + "\n"
    } else {
        new_content
    };
    if new_content != content {
        fs::write(path, new_content)?;
    }
    Ok(())
}

fn count_functions_in_source(source: &Path) -> usize {
    if !source.exists() {
        return 0;
    }
    let Ok(content) = fs::read_to_string(source) else {
        return 0;
    };
    crate::sync::extract_signatures(&content).len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_project(tmp: &tempfile::TempDir) -> std::path::PathBuf {
        let root = tmp.path().to_path_buf();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("include")).unwrap();
        fs::write(
            root.join("src/main.c"),
            "#include <stdio.h>\n\nint main(void) {\n    return 0;\n}\n",
        ).unwrap();
        root
    }

    #[test]
    fn add_creates_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = make_project(&tmp);
        add_module(&root, "parser").unwrap();
        assert!(root.join("include/parser.h").exists());
        assert!(root.join("src/parser.c").exists());
    }

    #[test]
    fn add_injects_include_into_main() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = make_project(&tmp);
        add_module(&root, "parser").unwrap();
        let main_c = fs::read_to_string(root.join("src/main.c")).unwrap();
        assert!(main_c.contains("#include \"parser.h\""));
    }

    #[test]
    fn add_duplicate_returns_error_and_no_double_include() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = make_project(&tmp);
        add_module(&root, "parser").unwrap();
        assert!(add_module(&root, "parser").is_err());
        let main_c = fs::read_to_string(root.join("src/main.c")).unwrap();
        assert_eq!(main_c.matches("#include \"parser.h\"").count(), 1);
    }

    #[test]
    fn add_stamps_spdx_from_project_license() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = make_project(&tmp);
        crate::config::ProjectConfig {
            license: Some("MIT".to_string()),
            ..Default::default()
        }
        .save_to(&root)
        .unwrap();
        add_module(&root, "parser").unwrap();
        let header = fs::read_to_string(root.join("include/parser.h")).unwrap();
        let source = fs::read_to_string(root.join("src/parser.c")).unwrap();
        assert!(header.starts_with("/* SPDX-License-Identifier: MIT */\n"));
        assert!(source.starts_with("/* SPDX-License-Identifier: MIT */\n"));
    }

    #[test]
    fn remove_deletes_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = make_project(&tmp);
        add_module(&root, "parser").unwrap();
        remove_module(&root, "parser").unwrap();
        assert!(!root.join("include/parser.h").exists());
        assert!(!root.join("src/parser.c").exists());
    }

    #[test]
    fn remove_strips_include_from_main() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = make_project(&tmp);
        add_module(&root, "parser").unwrap();
        remove_module(&root, "parser").unwrap();
        let main_c = fs::read_to_string(root.join("src/main.c")).unwrap();
        assert!(!main_c.contains("#include \"parser.h\""));
    }
}
