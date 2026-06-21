//! Header synchronisation — extracts function signatures from `.c` files and
//! regenerates the corresponding `.h` files.
//!
//! The `SYNC_IGNORE` block in a header is preserved verbatim across regeneration,
//! allowing user-written typedefs, structs, and `#define`s to coexist with
//! auto-generated prototypes.

use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::error::{NewcError, Result};

// Match a C function definition: return-type name(params) on one line, followed by '{' on the
// very next line (Allman brace style enforced by newc's generated code).
static FUNC_DEF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([a-zA-Z][^;{#\n]*\([^;{\n]*\))\s*\n\{").unwrap()
});

/// A parsed function extracted from a `.c` file.
#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    /// C function name (e.g. `"read_int"`).
    pub name: String,
    /// Normalised function signature without trailing semicolon.
    pub signature: String,
    /// Preceding block comment (`/* … */`), or empty if none.
    pub comment: String,
    /// Raw body text including the outer braces.
    pub body: String,
}

/// Extract all function implementations from a `.c` source string.
///
/// Recognises Allman-style brace placement (`{` on its own line). Returns the
/// preceding block comment, normalised signature, and body for each function found.
pub fn extract_function_implementations(source: &str) -> Vec<ExtractedFunction> {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Collect preceding comment block (/* ... */)
        let mut comment_lines: Vec<&str> = Vec::new();
        let mut j = i;

        // Skip blank lines before comment
        while j < lines.len() && lines[j].trim().is_empty() {
            j += 1;
        }
        if j >= lines.len() {
            break;
        }

        // Gather comment block
        if lines[j].trim().starts_with("/*") {
            let start = j;
            while j < lines.len() && !lines[j].contains("*/") {
                comment_lines.push(lines[j]);
                j += 1;
            }
            if j < lines.len() {
                comment_lines.push(lines[j]); // closing */
                j += 1;
            }

            // Skip blank lines after comment
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }

            if j >= lines.len() {
                i = start + 1;
                continue;
            }
        }

        // Check if next line looks like a function signature
        let sig_line = lines[j];
        if !sig_line.is_empty()
            && !sig_line.trim().starts_with("//")
            && !sig_line.trim().starts_with("*")
            && !sig_line.trim().starts_with("#")
            && !sig_line.contains(';')
            && sig_line.contains('(')
            && sig_line.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false)
        {
            // Peek: next non-blank line must be `{`
            let mut k = j + 1;
            while k < lines.len() && lines[k].trim().is_empty() {
                k += 1;
            }
            if k < lines.len() && lines[k].trim() == "{" {
                let sig = sig_line.split_whitespace().collect::<Vec<_>>().join(" ");
                // Extract function name from signature
                let name = extract_name_from_sig(&sig).unwrap_or_default();
                if !name.is_empty() {
                    // Collect body (from `{` to matching `}`)
                    let mut body_lines: Vec<&str> = Vec::new();
                    let mut depth: i32 = 0;
                    let mut m = k;
                    while m < lines.len() {
                        body_lines.push(lines[m]);
                        depth += lines[m].chars().filter(|&c| c == '{').count() as i32;
                        depth -= lines[m].chars().filter(|&c| c == '}').count() as i32;
                        m += 1;
                        if depth <= 0 {
                            break;
                        }
                    }
                    result.push(ExtractedFunction {
                        name,
                        signature: sig,
                        comment: comment_lines.join("\n"),
                        body: body_lines.join("\n"),
                    });
                    i = m;
                    continue;
                }
            }
        }

        // Nothing matched — advance past the comment block or current line
        if !comment_lines.is_empty() {
            i = j;
        } else {
            i += 1;
        }
    }

    result
}

fn extract_name_from_sig(sig: &str) -> Option<String> {
    let before_paren = sig.split('(').next()?;
    let name = before_paren.split_whitespace().last()?.trim_start_matches('*');
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty() {
        Some(name.to_string())
    } else {
        None
    }
}

/// Extract only the function signatures (no bodies) from a `.c` source string.
///
/// Uses the [`FUNC_DEF`] regex; requires Allman brace style (opening `{` on its
/// own line). Whitespace in each signature is normalised.
pub fn extract_signatures(source: &str) -> Vec<String> {
    FUNC_DEF
        .captures_iter(source)
        .map(|c| {
            // Normalise whitespace in multi-line signatures (shouldn't happen with Allman style
            // but guard anyway)
            c[1].split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect()
}

/// Regenerate `include/<name>.h` from the function signatures in `src/<name>.c`.
///
/// The `SYNC_IGNORE` block from the existing header is preserved. The guard macro
/// is derived from the module name (uppercased + `_H`).
///
/// # Errors
/// Returns an error if `src/<name>.c` does not exist, contains no functions, or
/// any file cannot be read or written.
pub fn sync_module(root: &Path, name: &str) -> Result<()> {
    let src = root.join("src").join(format!("{name}.c"));
    let hdr = root.join("include").join(format!("{name}.h"));

    if !src.exists() {
        return Err(NewcError::Other(format!("{} not found", src.display())));
    }

    let source_content = fs::read_to_string(&src)?;
    let funcs = extract_function_implementations(&source_content);

    if funcs.is_empty() {
        return Err(NewcError::Other(format!(
            "No functions found in {}, skipping.",
            src.display()
        )));
    }

    let guard = name.to_uppercase() + "_H";
    let preserved = extract_preserved(&hdr);

    // Carry Doxygen-style comments (/** ... */) forward into the regenerated
    // header above each prototype; plain comments are left in the .c file only.
    let protos: String = funcs
        .iter()
        .map(|f| {
            let sig = f.signature.split_whitespace().collect::<Vec<_>>().join(" ");
            if f.comment.trim_start().starts_with("/**") {
                format!("{}\n{sig};", f.comment)
            } else {
                format!("{sig};")
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let new_header = if preserved.trim().is_empty() {
        format!(
            "#ifndef {guard}\n#define {guard}\n\n{protos}\n\n#endif\n"
        )
    } else {
        format!(
            "#ifndef {guard}\n#define {guard}\n\n/* SYNC_IGNORE_START */\n{preserved}\n/* SYNC_IGNORE_END */\n\n{protos}\n\n#endif\n"
        )
    };

    fs::write(&hdr, new_header)?;
    Ok(())
}

/// Sync all module headers in the project.
///
/// Iterates every `.h` file in `include/` and calls [`sync_module`] for each.
/// Errors from individual modules are collected as warning strings rather than
/// aborting the entire operation.
///
/// # Returns
/// A list of status messages (successes followed by warnings).
///
/// # Errors
/// Returns an IO error only if `include/` cannot be read.
pub fn sync_all(root: &Path) -> Result<Vec<String>> {
    let include_dir = root.join("include");
    let mut synced = Vec::new();
    let mut errors = Vec::new();

    let mut names: Vec<String> = fs::read_dir(&include_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("h"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .collect();
    names.sort();

    for name in names {
        match sync_module(root, &name) {
            Ok(()) => synced.push(format!("Synced include/{name}.h")),
            Err(e) => errors.push(format!("Warning: {e}")),
        }
    }

    synced.extend(errors);
    Ok(synced)
}

/// Replace a function's implementation in a `.c` source file.
///
/// Removes the old definition (comment + signature + body) and appends `new_impl`
/// at the end of the file.
///
/// # Errors
/// Returns an IO error if the file cannot be read or written.
pub fn update_function_in_source(src: &Path, fname: &str, new_impl: &str) -> Result<()> {
    // Remove old implementation
    crate::analysis::remove_function_from_source_pub(src, fname)?;
    // Append new implementation
    let mut content = fs::read_to_string(src)?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(new_impl.trim_end());
    content.push('\n');
    fs::write(src, content)?;
    Ok(())
}

fn extract_preserved(hdr: &Path) -> String {
    let Ok(content) = fs::read_to_string(hdr) else {
        return String::new();
    };

    let mut inside = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        if line.contains("SYNC_IGNORE_START") {
            inside = true;
            continue;
        }
        if line.contains("SYNC_IGNORE_END") {
            inside = false;
            continue;
        }
        if inside {
            lines.push(line);
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allman(sig: &str, body: &str) -> String {
        format!("{sig}\n{{\n{body}\n}}\n")
    }

    #[test]
    fn extract_single_signature() {
        let code = allman("int add(int a, int b)", "    return a + b;");
        let sigs = extract_signatures(&code);
        assert_eq!(sigs.len(), 1);
        assert!(sigs[0].contains("add"));
    }

    #[test]
    fn extract_multiple_signatures() {
        let code = format!(
            "{}\n\n{}",
            allman("int foo(void)", "    return 0;"),
            allman("void bar(int x)", "    (void)x;")
        );
        let sigs = extract_signatures(&code);
        assert_eq!(sigs.len(), 2);
    }

    #[test]
    fn extract_empty_source() {
        assert!(extract_signatures("").is_empty());
    }

    #[test]
    fn extract_preserves_signature_text() {
        let code = allman("char *to_upper(char *s)", "    return s;");
        let sigs = extract_signatures(&code);
        assert_eq!(sigs.len(), 1);
        assert!(sigs[0].contains("to_upper"));
        assert!(sigs[0].contains("char"));
    }
}
