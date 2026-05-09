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

/// A parsed function extracted from a .c file.
#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub signature: String,
    pub comment: String,
    pub body: String,
}

/// Extract full function implementations from a .c file source string.
/// Returns comment block + signature + body for each function found.
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

pub fn sync_module(root: &Path, name: &str) -> Result<()> {
    let src = root.join("src").join(format!("{name}.c"));
    let hdr = root.join("include").join(format!("{name}.h"));

    if !src.exists() {
        return Err(NewcError::Other(format!("{} not found", src.display())));
    }

    let source_content = fs::read_to_string(&src)?;
    let sigs = extract_signatures(&source_content);

    if sigs.is_empty() {
        return Err(NewcError::Other(format!(
            "No functions found in {}, skipping.",
            src.display()
        )));
    }

    let guard = name.to_uppercase() + "_H";
    let preserved = extract_preserved(&hdr);

    let protos: String = sigs
        .iter()
        .map(|s| format!("{s};"))
        .collect::<Vec<_>>()
        .join("\n");

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

/// Replace a function's implementation in a .c source file.
/// Removes the old definition (comment + signature + body) and appends the new one.
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
