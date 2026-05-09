use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{NewcError, Result};
use crate::sync::extract_signatures;

#[derive(Debug, Clone)]
pub struct UnreachableFunc {
    pub name: String,
    pub source: PathBuf,
}

pub fn check(root: &Path) -> Result<Vec<UnreachableFunc>> {
    let func_map = collect_module_functions(root)?;
    if func_map.is_empty() {
        return Ok(Vec::new());
    }
    let reachable = bfs_reachability(root, &func_map)?;
    let mut unreachable: Vec<UnreachableFunc> = func_map
        .into_iter()
        .filter(|(name, _)| !reachable.contains(name))
        .map(|(name, source)| UnreachableFunc { name, source })
        .collect();
    unreachable.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(unreachable)
}

pub fn tidy(root: &Path, to_remove: &[String]) -> Result<Vec<String>> {
    let func_map = collect_module_functions(root)?;
    let mut log = Vec::new();

    let mut affected_modules: HashSet<String> = HashSet::new();

    for fname in to_remove {
        let Some(src_path) = func_map.get(fname) else {
            continue;
        };
        let module = src_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let hdr_path = root.join("include").join(format!("{module}.h"));

        remove_function_from_source(src_path, fname)?;
        remove_prototype_from_header(&hdr_path, fname)?;

        affected_modules.insert(module.clone());
        log.push(format!("Removed {fname}."));
    }

    // Delete modules that became empty
    for module in &affected_modules {
        let src = root.join("src").join(format!("{module}.c"));
        let hdr = root.join("include").join(format!("{module}.h"));
        if src.exists() {
            let content = fs::read_to_string(&src)?;
            if extract_signatures(&content).is_empty() {
                fs::remove_file(&src)?;
                if hdr.exists() {
                    fs::remove_file(&hdr)?;
                }
                // Remove #include from all .c files
                let src_dir = root.join("src");
                if let Ok(entries) = fs::read_dir(&src_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.extension().and_then(|x| x.to_str()) == Some("c") {
                            remove_include_line(&path, module)?;
                        }
                    }
                }
                log.push(format!("Removed empty module '{module}'."));
            }
        }
    }

    Ok(log)
}

fn collect_module_functions(root: &Path) -> Result<HashMap<String, PathBuf>> {
    let src_dir = root.join("src");
    let mut map: HashMap<String, PathBuf> = HashMap::new();

    let mut entries: Vec<_> = fs::read_dir(&src_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("c")
                && e.file_name() != "main.c"
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        for sig in extract_signatures(&content) {
            if let Some(name) = extract_func_name(&sig) {
                map.insert(name, path.clone());
            }
        }
    }

    if map.is_empty() {
        return Err(NewcError::NoModules);
    }

    Ok(map)
}

fn bfs_reachability(
    root: &Path,
    func_map: &HashMap<String, PathBuf>,
) -> Result<HashSet<String>> {
    let main_c = root.join("src").join("main.c");
    let main_content = if main_c.exists() {
        fs::read_to_string(&main_c)?
    } else {
        String::new()
    };

    let mut reachable: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    // Seed: find calls to module functions anywhere in main.c
    for fname in func_map.keys() {
        if is_called_in(&main_content, fname) {
            queue.push_back(fname.clone());
        }
    }

    while let Some(fname) = queue.pop_front() {
        if reachable.contains(&fname) {
            continue;
        }
        reachable.insert(fname.clone());

        let Some(src_path) = func_map.get(&fname) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(src_path) else {
            continue;
        };

        // Find body of this function and search for calls to other module functions
        let body = extract_function_body(&content, &fname);
        for other in func_map.keys() {
            if !reachable.contains(other) && is_called_in(&body, other) {
                queue.push_back(other.clone());
            }
        }
    }

    Ok(reachable)
}

fn is_called_in(content: &str, fname: &str) -> bool {
    // Matches fname( anywhere that isn't a definition line (no return type at start of line)
    let pattern = format!("{fname}(");
    for line in content.lines() {
        let trimmed = line.trim();
        // Skip the definition line itself (starts with return type token)
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        if trimmed.contains(&pattern) && !is_definition_line(trimmed, fname) {
            return true;
        }
    }
    false
}

fn is_definition_line(line: &str, fname: &str) -> bool {
    // A definition line starts with the return type (no leading whitespace after trim)
    // and the function name follows directly with no assignment or call context
    line.starts_with(fname) || {
        // e.g. "int fname(" or "void fname("
        let pat = format!(" {fname}(");
        line.contains(&pat) && !line.contains('=') && !line.contains("return")
    }
}

fn extract_function_body(content: &str, fname: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_target = false;
    let mut brace_depth: i32 = 0;
    let mut body = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if !in_target {
            // Look for the function signature line
            if line.contains(fname) && line.contains('(') && !line.trim_start().starts_with("/*")
                && !line.trim_start().starts_with("*") && !line.contains(';')
            {
                // Confirm next non-empty line is '{'
                if lines.get(i + 1).map(|l| l.trim()) == Some("{") {
                    in_target = true;
                }
            }
        } else {
            body.push(*line);
            brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;
            if brace_depth <= 0 && body.len() > 1 {
                break;
            }
        }
    }

    body.join("\n")
}

fn extract_func_name(sig: &str) -> Option<String> {
    // Match: "type name(" → extract name
    // Pattern: last word-token before '('
    let before_paren = sig.split('(').next()?;
    let name = before_paren.split_whitespace().last()?.trim_start_matches('*');
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}

pub fn remove_function_from_source_pub(src: &Path, fname: &str) -> Result<()> {
    remove_function_from_source(src, fname)
}

fn remove_function_from_source(src: &Path, fname: &str) -> Result<()> {
    let content = fs::read_to_string(src)?;
    let lines: Vec<&str> = content.lines().collect();

    #[derive(PartialEq)]
    enum State {
        Normal,
        Comment,
        PostComment,
        SkipSig,
        SkipBody,
    }

    let mut state = State::Normal;
    let mut buf: Vec<&str> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    let mut brace_depth: i32 = 0;

    for line in &lines {
        match state {
            State::Normal => {
                if line.trim_start().starts_with("/*") {
                    state = State::Comment;
                    buf.clear();
                    buf.push(line);
                } else {
                    out.push(line.to_string());
                }
            }
            State::Comment => {
                buf.push(line);
                if line.contains("*/") {
                    state = State::PostComment;
                }
            }
            State::PostComment => {
                // Does this line contain the target function name as a definition?
                if is_func_def_line(line, fname) {
                    buf.clear();
                    state = State::SkipSig;
                } else {
                    // Flush buffered comment to output, process this line normally
                    for b in &buf {
                        out.push(b.to_string());
                    }
                    buf.clear();
                    if line.trim_start().starts_with("/*") {
                        state = State::Comment;
                        buf.push(line);
                    } else {
                        state = State::Normal;
                        out.push(line.to_string());
                    }
                }
            }
            State::SkipSig => {
                if line.trim() == "{" {
                    state = State::SkipBody;
                    brace_depth = 1;
                }
                // else: still in multi-line signature, keep skipping
            }
            State::SkipBody => {
                brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
                brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;
                if brace_depth <= 0 {
                    state = State::Normal;
                }
            }
        }
    }

    // Flush any remaining buffered comment
    if !buf.is_empty() {
        for b in &buf {
            out.push(b.to_string());
        }
    }

    let new_content = out.join("\n");
    let new_content = if content.ends_with('\n') {
        new_content + "\n"
    } else {
        new_content
    };
    fs::write(src, new_content)?;
    Ok(())
}

fn is_func_def_line(line: &str, fname: &str) -> bool {
    let trimmed = line.trim();
    // Must contain fname( and not be a comment or declaration (no ';')
    if !trimmed.contains(fname) || trimmed.contains(';') {
        return false;
    }
    let pat = format!("{fname}(");
    trimmed.contains(&pat)
        && (trimmed.starts_with(fname)
            || trimmed.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false))
}

fn remove_prototype_from_header(hdr: &Path, fname: &str) -> Result<()> {
    if !hdr.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(hdr)?;
    let pattern = format!("{fname}(");
    let new_content: String = content
        .lines()
        .filter(|line| {
            // Remove the prototype line and its preceding comment line
            let trimmed = line.trim();
            !(trimmed.contains(&pattern) && trimmed.ends_with(';'))
        })
        .collect::<Vec<_>>()
        .join("\n");
    // Remove dangling comment line that was directly above the prototype
    let new_content = remove_orphan_comment_before_next_proto(&new_content, fname);
    let new_content = if content.ends_with('\n') {
        new_content + "\n"
    } else {
        new_content
    };
    fs::write(hdr, new_content)?;
    Ok(())
}

fn remove_orphan_comment_before_next_proto(content: &str, _removed_fname: &str) -> String {
    // Simple pass: collapse multiple consecutive blank lines to one
    let mut out = Vec::new();
    let mut blank_count = 0;
    for line in content.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                out.push(line.to_string());
            }
        } else {
            blank_count = 0;
            out.push(line.to_string());
        }
    }
    out.join("\n")
}

fn remove_include_line(path: &Path, module_name: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let target = format!("#include \"{module_name}.h\"");
    let new_content: String = content
        .lines()
        .filter(|l| l.trim() != target.trim())
        .collect::<Vec<_>>()
        .join("\n");
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
