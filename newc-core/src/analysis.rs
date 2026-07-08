//! Dead-code detection and removal for C projects.
//!
//! Performs BFS reachability analysis starting from `main()` to find functions
//! in module `.c` files that are never called, and can surgically remove them
//! from both the source and the corresponding header.
//!
//! Analysis is text-based, not a real C parser: it assumes the Allman brace
//! style and one-line signatures that newc's own generated code uses. Calls
//! are detected with a word-boundary regex (`\bname\s*\(`), so identifiers
//! that merely contain a function's name (`do_foo` vs `foo`) don't count.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::error::{NewcError, Result};
use crate::sync::extract_signatures;

/// A module function that cannot be reached from `main()`.
#[derive(Debug, Clone)]
pub struct UnreachableFunc {
    /// Fully-qualified C function name.
    pub name: String,
    /// Path to the `.c` file that contains the definition.
    pub source: PathBuf,
    /// True if the definition is `static` (file scope).
    pub is_static: bool,
}

#[derive(Debug, Clone)]
struct FuncInfo {
    source: PathBuf,
    is_static: bool,
}

fn call_regex(fname: &str) -> Regex {
    Regex::new(&format!(r"\b{}\s*\(", regex::escape(fname))).unwrap()
}

/// Return all module functions that are unreachable from `main()`.
///
/// # Errors
/// Returns [`NewcError::NoModules`] if the project has no module source files,
/// or an IO error if any source file cannot be read.
pub fn check(root: &Path) -> Result<Vec<UnreachableFunc>> {
    let func_map = collect_module_functions(root)?;
    if func_map.is_empty() {
        return Ok(Vec::new());
    }

    // Read each module source once; the BFS and static checks below reuse these.
    let mut sources: HashMap<PathBuf, String> = HashMap::new();
    for info in func_map.values() {
        if !sources.contains_key(&info.source) {
            sources.insert(info.source.clone(), fs::read_to_string(&info.source)?);
        }
    }
    let regexes: HashMap<String, Regex> = func_map
        .keys()
        .map(|n| (n.clone(), call_regex(n)))
        .collect();

    let reachable = bfs_reachability(root, &func_map, &sources, &regexes)?;
    let mut unreachable: Vec<UnreachableFunc> = func_map
        .iter()
        .filter(|(name, _)| !reachable.contains(*name))
        .filter(|(name, info)| {
            // Keep a `static` fn that is called anywhere in its own file: file-scope
            // linkage means the caller may be outside the reachability graph.
            !(info.is_static
                && sources
                    .get(&info.source)
                    .is_some_and(|c| is_called_in(c, &regexes[*name])))
        })
        .map(|(name, info)| UnreachableFunc {
            name: name.clone(),
            source: info.source.clone(),
            is_static: info.is_static,
        })
        .collect();
    unreachable.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(unreachable)
}

/// Remove the specified functions from their source files and headers.
///
/// Also deletes any module that becomes empty (no remaining functions) and
/// removes the corresponding `#include` line from every `.c` file in the project.
///
/// # Returns
/// A log of actions taken, one entry per removal or deleted module.
///
/// # Errors
/// Propagates IO errors from reading or writing source files.
pub fn tidy(root: &Path, to_remove: &[String]) -> Result<Vec<String>> {
    let func_map = collect_module_functions(root)?;
    let mut log = Vec::new();

    let mut affected_modules: HashSet<String> = HashSet::new();

    for fname in to_remove {
        let Some(src_path) = func_map.get(fname).map(|i| &i.source) else {
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

fn collect_module_functions(root: &Path) -> Result<HashMap<String, FuncInfo>> {
    let src_dir = root.join("src");
    let mut map: HashMap<String, FuncInfo> = HashMap::new();

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
                let is_static = sig.starts_with("static ");
                map.insert(name, FuncInfo { source: path.clone(), is_static });
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
    func_map: &HashMap<String, FuncInfo>,
    sources: &HashMap<PathBuf, String>,
    regexes: &HashMap<String, Regex>,
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
        if is_called_in(&main_content, &regexes[fname]) {
            queue.push_back(fname.clone());
        }
    }

    while let Some(fname) = queue.pop_front() {
        if reachable.contains(&fname) {
            continue;
        }
        reachable.insert(fname.clone());

        let Some(info) = func_map.get(&fname) else {
            continue;
        };
        let Some(content) = sources.get(&info.source) else {
            continue;
        };

        // Find body of this function and search for calls to other module functions
        let body = extract_function_body(content, &regexes[&fname]);
        for other in func_map.keys() {
            if !reachable.contains(other) && is_called_in(&body, &regexes[other]) {
                queue.push_back(other.clone());
            }
        }
    }

    Ok(reachable)
}

fn is_called_in(content: &str, re: &Regex) -> bool {
    // Matches `\bname\s*\(` anywhere that isn't the definition line itself
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        if let Some(m) = re.find(trimmed)
            && !is_definition_line(trimmed, m.start())
        {
            return true;
        }
    }
    false
}

fn is_definition_line(line: &str, name_start: usize) -> bool {
    // Calls always end with ';'; definitions never do (body follows on next line)
    if line.ends_with(';') {
        return false;
    }
    if line.contains('=') || line.contains("return") {
        return false;
    }
    // A definition has return-type tokens before the name: "int foo(" / "char *foo("
    let prefix = line[..name_start].trim_end();
    prefix
        .chars()
        .next_back()
        .map(|c| c.is_ascii_alphanumeric() || c == '_' || c == '*')
        .unwrap_or(false)
}

fn extract_function_body(content: &str, re: &Regex) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_target = false;
    let mut brace_depth: i32 = 0;
    let mut body = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if !in_target {
            // Look for the function signature line
            if re.is_match(line) && line.contains('(') && !line.trim_start().starts_with("/*")
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

/// Public wrapper around the internal function-removal routine.
///
/// Strips the block comment immediately preceding `fname`, its signature, and
/// its entire body from `src`. No-ops gracefully if `fname` is not found.
///
/// # Errors
/// Returns an IO error if `src` cannot be read or written.
pub fn remove_function_from_source_pub(src: &Path, fname: &str) -> Result<()> {
    remove_function_from_source(src, fname)
}

fn remove_function_from_source(src: &Path, fname: &str) -> Result<()> {
    let re = call_regex(fname);
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
                } else if is_func_def_line(line, &re) {
                    // Definition without a preceding comment block
                    state = State::SkipSig;
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
                if is_func_def_line(line, &re) {
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

fn is_func_def_line(line: &str, re: &Regex) -> bool {
    let trimmed = line.trim();
    // Must contain the name at a word boundary and not be a declaration (no ';')
    if trimmed.contains(';') || !re.is_match(trimmed) {
        return false;
    }
    trimmed.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
}

fn remove_prototype_from_header(hdr: &Path, fname: &str) -> Result<()> {
    if !hdr.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(hdr)?;
    let re = call_regex(fname);
    let new_content: String = content
        .lines()
        .filter(|line| {
            // Remove the prototype line and its preceding comment line
            let trimmed = line.trim();
            !(re.is_match(trimmed) && trimmed.ends_with(';'))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Write a minimal project: main.c with the given body, plus (name, content) modules.
    fn write_project(tmp: &tempfile::TempDir, main_body: &str, modules: &[(&str, &str)]) -> PathBuf {
        let root = tmp.path().to_path_buf();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("include")).unwrap();
        fs::write(
            root.join("src/main.c"),
            format!("int main(void)\n{{\n{main_body}\n    return 0;\n}}\n"),
        )
        .unwrap();
        for (name, content) in modules {
            fs::write(root.join("src").join(format!("{name}.c")), content).unwrap();
        }
        root
    }

    fn allman(sig: &str, body: &str) -> String {
        format!("{sig}\n{{\n{body}\n}}\n")
    }

    #[test]
    fn call_to_do_foo_does_not_reach_foo() {
        let tmp = tempfile::TempDir::new().unwrap();
        let module = format!(
            "{}\n{}",
            allman("int foo(void)", "    return 1;"),
            allman("int do_foo(void)", "    return 2;")
        );
        let root = write_project(&tmp, "    do_foo();", &[("utils", &module)]);
        let unreachable = check(&root).unwrap();
        assert_eq!(unreachable.len(), 1);
        assert_eq!(unreachable[0].name, "foo");
    }

    #[test]
    fn call_with_space_before_paren_counts() {
        let tmp = tempfile::TempDir::new().unwrap();
        let module = allman("int foo(void)", "    return 1;");
        let root = write_project(&tmp, "    foo ();", &[("utils", &module)]);
        assert!(check(&root).unwrap().is_empty());
    }

    #[test]
    fn transitive_calls_are_reachable() {
        let tmp = tempfile::TempDir::new().unwrap();
        let module = format!(
            "{}\n{}",
            allman("int first(void)", "    return second();"),
            allman("int second(void)", "    return 2;")
        );
        let root = write_project(&tmp, "    first();", &[("utils", &module)]);
        assert!(check(&root).unwrap().is_empty());
    }

    #[test]
    fn static_called_in_own_file_is_kept() {
        let tmp = tempfile::TempDir::new().unwrap();
        // `unused` is unreachable but calls the static helper — helper must not be listed.
        let module = format!(
            "{}\n{}",
            allman("static int helper(void)", "    return 1;"),
            allman("int unused(void)", "    return helper();")
        );
        let root = write_project(&tmp, "    ;", &[("utils", &module)]);
        let unreachable = check(&root).unwrap();
        let names: Vec<&str> = unreachable.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["unused"]);
    }

    #[test]
    fn uncalled_static_is_flagged_as_static() {
        let tmp = tempfile::TempDir::new().unwrap();
        let module = allman("static int orphan(void)", "    return 1;");
        let root = write_project(&tmp, "    ;", &[("utils", &module)]);
        let unreachable = check(&root).unwrap();
        assert_eq!(unreachable.len(), 1);
        assert_eq!(unreachable[0].name, "orphan");
        assert!(unreachable[0].is_static);
    }

    #[test]
    fn tidy_removes_function_and_prototype() {
        let tmp = tempfile::TempDir::new().unwrap();
        let module = format!(
            "{}\n{}",
            allman("int used(void)", "    return 1;"),
            allman("int dead(void)", "    return 2;")
        );
        let root = write_project(&tmp, "    used();", &[("utils", &module)]);
        fs::write(
            root.join("include/utils.h"),
            "#ifndef UTILS_H\n#define UTILS_H\n\nint used(void);\n\nint dead(void);\n\n#endif\n",
        )
        .unwrap();

        let log = tidy(&root, &["dead".to_string()]).unwrap();
        assert!(log.iter().any(|l| l.contains("dead")));
        let src = fs::read_to_string(root.join("src/utils.c")).unwrap();
        assert!(!src.contains("dead"));
        assert!(src.contains("used"));
        let hdr = fs::read_to_string(root.join("include/utils.h")).unwrap();
        assert!(!hdr.contains("dead"));
    }
}
