//! Parser for Valgrind's `--xml=yes` memcheck report.
//!
//! Hand-rolled rather than pulling an XML crate or regex — Valgrind's report
//! has a flat, predictable structure, so plain substring search for each
//! `<tag>...</tag>` is enough.

/// Return the text between the first `<tag>` and the next `</tag>` in `block`.
fn tag_content<'a>(block: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&close)?;
    Some(&block[start..start + end])
}

/// A single Valgrind error (memory leak, invalid read/write, uninitialised use, etc).
#[derive(Debug, Clone)]
pub struct ValgrindError {
    /// Machine-readable kind, e.g. `"Leak_DefinitelyLost"`, `"InvalidRead"`.
    pub kind: String,
    /// Human-readable summary, e.g. `"Invalid read of size 4"`.
    pub text: String,
    /// Bytes leaked, if this error is a leak report.
    pub leaked_bytes: Option<u64>,
    /// Source file of the first frame with file/line info, if any.
    pub file: Option<String>,
    /// Source line of the first frame with file/line info, if any.
    pub line: Option<usize>,
}

/// Summary counts across all parsed errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValgrindSummary {
    pub error_count: usize,
    pub total_leaked_bytes: u64,
}

/// Parse a Valgrind `--xml=yes` report into structured errors.
pub fn parse(xml: &str) -> Vec<ValgrindError> {
    let mut errors = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<error>") {
        let body_start = start + "<error>".len();
        let Some(len) = rest[body_start..].find("</error>") else { break };
        let block = &rest[body_start..body_start + len];

        errors.push(ValgrindError {
            kind: tag_content(block, "kind").unwrap_or_default().to_string(),
            text: tag_content(block, "text").unwrap_or_default().to_string(),
            leaked_bytes: tag_content(block, "leakedbytes").and_then(|s| s.parse().ok()),
            file: tag_content(block, "file").map(str::to_string),
            line: tag_content(block, "line").and_then(|s| s.parse().ok()),
        });

        rest = &rest[body_start + len + "</error>".len()..];
    }
    errors
}

/// Summarize error count and total leaked bytes across all parsed errors.
pub fn summarize(errors: &[ValgrindError]) -> ValgrindSummary {
    ValgrindSummary {
        error_count: errors.len(),
        total_leaked_bytes: errors.iter().filter_map(|e| e.leaked_bytes).sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
<valgrindoutput>
<error>
<kind>Leak_DefinitelyLost</kind>
<xwhat>
<text>40 bytes in 1 blocks are definitely lost</text>
<leakedbytes>40</leakedbytes>
</xwhat>
<stack>
<frame><file>module.c</file><line>12</line></frame>
</stack>
</error>
<error>
<kind>InvalidRead</kind>
<text>Invalid read of size 4</text>
<stack>
<frame><file>module.c</file><line>20</line></frame>
</stack>
</error>
</valgrindoutput>
"#;

    #[test]
    fn parses_two_errors() {
        let errors = parse(SAMPLE);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].kind, "Leak_DefinitelyLost");
        assert_eq!(errors[0].leaked_bytes, Some(40));
        assert_eq!(errors[0].file.as_deref(), Some("module.c"));
        assert_eq!(errors[0].line, Some(12));
        assert_eq!(errors[1].kind, "InvalidRead");
    }

    #[test]
    fn summary_totals() {
        let summary = summarize(&parse(SAMPLE));
        assert_eq!(summary.error_count, 2);
        assert_eq!(summary.total_leaked_bytes, 40);
    }
}
