use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{NewcError, Result};

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub header: PathBuf,
    pub source: PathBuf,
    pub function_count: usize,
}

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

pub fn add_module(root: &Path, name: &str) -> Result<()> {
    let header = root.join("include").join(format!("{name}.h"));
    let source = root.join("src").join(format!("{name}.c"));

    if header.exists() || source.exists() {
        return Err(NewcError::ModuleExists(name.to_string()));
    }

    let guard = name.to_uppercase() + "_H";
    let header_content = format!(
        "#ifndef {guard}\n#define {guard}\n\n/* SYNC_IGNORE_START */\n\n/* SYNC_IGNORE_END */\n\n#endif\n"
    );
    let source_content = format!(
        "#include <stdio.h>\n#include \"{name}.h\"\n\n/*\n * Function:\n *     ...\n * Input:\n *     ...\n * Output:\n *     ...\n * Algorithm:\n *     ...\n */\n\n"
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
        .last();

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
