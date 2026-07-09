//! Doxygen-style documentation stub generation.
//!
//! [`generate_stub`] builds a `/** @brief ... */` comment block from a C
//! function signature. [`insert_stub`] writes that block above a function's
//! definition in its `.c` file, then re-syncs the header so the comment is
//! carried forward above the matching prototype (see [`crate::sync::sync_module`]).

use std::path::Path;

use crate::error::{NewcError, Result};
use crate::sync::{extract_function_implementations, sync_module};

/// Split a parameter list on top-level commas (no nested parens expected in C param lists).
fn split_params(params: &str) -> Vec<&str> {
    params
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Extract the parameter name from a single parameter declaration, e.g.
/// `"const char *prompt"` -> `"prompt"`, `"int n"` -> `"n"`.
fn param_name(param: &str) -> Option<&str> {
    let trimmed = param
        .trim_end_matches(']')
        .trim_end_matches(|c: char| c.is_ascii_digit());
    let trimmed = trimmed.trim_end_matches('[');
    let name = trimmed
        .rsplit(|c: char| c == '*' || c.is_whitespace())
        .next()?;
    if name.is_empty() || name == "void" {
        None
    } else {
        Some(name)
    }
}

/// Build a `/** @brief ... */` Doxygen stub for a C function signature.
///
/// `signature` is the normalised text before the trailing `{`, e.g.
/// `"int read_int(const char *prompt)"`.
pub fn generate_stub(signature: &str) -> String {
    let sig = signature.trim();
    let open = sig.find('(').unwrap_or(sig.len());
    let close = sig.rfind(')').unwrap_or(sig.len());
    let head = &sig[..open];
    let params_str = if close > open {
        &sig[open + 1..close]
    } else {
        ""
    };

    let return_type = head
        .rsplit_once(|c: char| c.is_whitespace() || c == '*')
        .map(|(rt, _)| rt.trim())
        .unwrap_or(head.trim());

    let mut out = String::from("/**\n * @brief \n");
    for param in split_params(params_str) {
        if let Some(name) = param_name(param) {
            out.push_str(&format!(" * @param {name} \n"));
        }
    }
    if return_type != "void" && !return_type.is_empty() {
        out.push_str(" * @return \n");
    }
    out.push_str(" */");
    out
}

/// A parsed `/** @brief ... @param ... @return ... */` Doxygen comment block.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedDoc {
    pub brief: String,
    /// `(name, description)` pairs, in declaration order.
    pub params: Vec<(String, String)>,
    pub returns: Option<String>,
}

/// Parse a Doxygen-style comment block into structured fields for display.
///
/// Returns `None` if `comment` is empty or not a `/** ... */` block.
pub fn parse_doc_comment(comment: &str) -> Option<ParsedDoc> {
    let trimmed = comment.trim();
    if !trimmed.starts_with("/**") {
        return None;
    }

    let mut doc = ParsedDoc::default();
    for raw_line in trimmed.lines() {
        let line = raw_line
            .trim()
            .trim_start_matches('*')
            .trim_start_matches("/**")
            .trim_end_matches("*/")
            .trim();
        if let Some(rest) = line.strip_prefix("@brief") {
            doc.brief = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("@param") {
            let rest = rest.trim();
            if let Some((name, desc)) = rest.split_once(char::is_whitespace) {
                doc.params.push((name.to_string(), desc.trim().to_string()));
            } else if !rest.is_empty() {
                doc.params.push((rest.to_string(), String::new()));
            }
        } else if let Some(rest) = line.strip_prefix("@return") {
            doc.returns = Some(rest.trim().to_string());
        }
    }
    Some(doc)
}

/// Insert a Doxygen stub above `fname`'s definition in `<module>.c`, then re-sync
/// the header so the same comment appears above the prototype.
///
/// # Errors
/// Returns an error if the function is not found, already has a comment, or on
/// any IO failure.
pub fn insert_stub(root: &Path, module: &str, fname: &str) -> Result<()> {
    let src_path = root.join("src").join(format!("{module}.c"));
    let content = std::fs::read_to_string(&src_path)?;

    let funcs = extract_function_implementations(&content);
    let func = funcs
        .iter()
        .find(|f| f.name == fname)
        .ok_or_else(|| NewcError::Other(format!("function '{fname}' not found in {module}")))?;

    if !func.comment.trim().is_empty() {
        return Err(NewcError::Other(format!("'{fname}' already has a comment")));
    }

    let old = format!("{}\n{}", func.signature, func.body);
    let stub = generate_stub(&func.signature);
    let new = format!("{stub}\n{old}");

    let updated = content.replacen(&old, &new, 1);
    std::fs::write(&src_path, updated)?;

    sync_module(root, module)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_with_params_and_return() {
        let stub = generate_stub("int read_int(const char *prompt)");
        assert!(stub.contains("@param prompt"));
        assert!(stub.contains("@return"));
    }

    #[test]
    fn stub_void_no_params() {
        let stub = generate_stub("void greet(void)");
        assert!(!stub.contains("@param"));
        assert!(!stub.contains("@return"));
    }

    #[test]
    fn parses_doc_comment() {
        let comment = "/**\n * @brief Reads an int\n * @param prompt the prompt text\n * @return the parsed int\n */";
        let doc = parse_doc_comment(comment).unwrap();
        assert_eq!(doc.brief, "Reads an int");
        assert_eq!(
            doc.params,
            vec![("prompt".to_string(), "the prompt text".to_string())]
        );
        assert_eq!(doc.returns, Some("the parsed int".to_string()));
    }

    #[test]
    fn non_doxygen_comment_returns_none() {
        assert!(parse_doc_comment("/* just a comment */").is_none());
        assert!(parse_doc_comment("").is_none());
    }

    #[test]
    fn stub_multiple_params() {
        let stub = generate_stub("int add(int a, int b)");
        assert!(stub.contains("@param a"));
        assert!(stub.contains("@param b"));
    }
}
