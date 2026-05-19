//! Parser for GCC/clang compiler diagnostic output.
//!
//! Matches lines of the form `file.c:12:5: error: message` and converts them
//! into structured [`Diagnostic`] records for display in the GUI build panel.

/// Parse GCC/clang diagnostic output into structured records.
/// Matches lines like: `file.c:12:5: error: message`

/// Severity level of a compiler diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub enum DiagKind {
    /// Fatal compilation error.
    Error,
    /// Non-fatal warning.
    Warning,
    /// Informational note (often attached to a prior error).
    Note,
}

/// A single structured compiler diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Source file name as reported by the compiler.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number, or `0` if the compiler omitted it.
    pub col: usize,
    pub kind: DiagKind,
    pub message: String,
}

impl Diagnostic {
    /// Returns `true` if this diagnostic is a hard error (not a warning or note).
    pub fn is_error(&self) -> bool {
        self.kind == DiagKind::Error
    }
}

/// Parse raw compiler output lines into a list of diagnostics.
///
/// Lines that do not match the GCC/clang format are silently skipped.
pub fn parse(lines: &[String]) -> Vec<Diagnostic> {
    let mut result = Vec::new();
    for line in lines {
        if let Some(d) = parse_line(line) {
            result.push(d);
        }
    }
    result
}

fn parse_line(line: &str) -> Option<Diagnostic> {
    // GCC/clang format: path:line:col: kind: message
    // Also handle: path:line: kind: message (no col)
    let mut parts = line.splitn(5, ':');
    let file = parts.next()?.trim().to_string();
    if file.is_empty() || file.starts_with(' ') { return None; }

    let line_no_str = parts.next()?.trim();
    let line_no: usize = line_no_str.parse().ok()?;

    let third = parts.next()?.trim();
    // Third could be col number or "error"/"warning"/"note"
    let (col, kind_str, message) = if let Ok(col) = third.parse::<usize>() {
        let kind_str = parts.next()?.trim().to_string();
        let msg = parts.next().unwrap_or("").trim().to_string();
        (col, kind_str, msg)
    } else {
        // No col — third is kind
        let msg = parts.next().unwrap_or("").trim().to_string();
        let remaining = parts.next().unwrap_or("").trim().to_string();
        let full_msg = if remaining.is_empty() { msg } else { format!("{msg} {remaining}") };
        (0, third.to_string(), full_msg)
    };

    let kind = match kind_str.as_str() {
        "error" => DiagKind::Error,
        "warning" => DiagKind::Warning,
        "note" => DiagKind::Note,
        _ => return None,
    };

    Some(Diagnostic { file, line: line_no, col, kind, message })
}
