//! `gcov` `.gcov` report parsing — line and per-function coverage percentages.
//!
//! Produced by the `coverage` Makefile/CMake target, which builds with
//! `-fprofile-arcs -ftest-coverage` (`--coverage` for CMake), runs the binary
//! once to generate `.gcda` data, then runs `gcov` to emit `<file>.c.gcov`
//! text reports next to the source.

use std::path::Path;

/// Coverage state of a single source line as reported by `gcov`.
#[derive(Debug, Clone, Copy)]
pub struct LineCoverage {
    pub line_no: usize,
    /// `None` if the line is not executable (comments, blank lines, braces).
    /// `Some(0)` means executable but never hit.
    pub count: Option<u64>,
}

/// Aggregate coverage summary over a set of lines.
#[derive(Debug, Clone, Copy, Default)]
pub struct CoverageSummary {
    pub covered: usize,
    pub total: usize,
}

impl CoverageSummary {
    pub fn percent(&self) -> f32 {
        if self.total == 0 { 0.0 } else { 100.0 * self.covered as f32 / self.total as f32 }
    }
}

/// Parse a `.gcov` text report into per-line coverage data.
///
/// Each line of the report has the form `<count>:<line_no>:<source>`, where
/// `count` is `-` (not executable), `#####` (executable, never hit), or a
/// hit count.
pub fn parse_gcov(content: &str) -> Vec<LineCoverage> {
    content
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, ':');
            let count_str = parts.next()?.trim();
            let line_no: usize = parts.next()?.trim().parse().ok()?;
            if line_no == 0 {
                return None; // header pseudo-line (Source:, Graph:, etc.)
            }
            let count = match count_str {
                "-" => None,
                "#####" | "=====" => Some(0),
                n => n.parse::<u64>().ok(),
            };
            Some(LineCoverage { line_no, count })
        })
        .collect()
}

/// Locate the `.gcov` report for `<module>.c` next to the source file and
/// summarize its overall line coverage. Returns `None` if no report exists
/// (coverage target hasn't been run yet).
pub fn module_summary(root: &Path, module: &str) -> Option<CoverageSummary> {
    let gcov_path = root.join("src").join(format!("{module}.c.gcov"));
    let content = std::fs::read_to_string(&gcov_path).ok()?;
    let lines = parse_gcov(&content);
    Some(summarize(&lines))
}

/// Summarize total/covered counts for a set of parsed lines.
pub fn summarize(lines: &[LineCoverage]) -> CoverageSummary {
    let mut summary = CoverageSummary::default();
    for l in lines {
        if let Some(count) = l.count {
            summary.total += 1;
            if count > 0 {
                summary.covered += 1;
            }
        }
    }
    summary
}

/// Per-function coverage breakdown, matching each function's body line range
/// (located via [`crate::sync::extract_function_implementations`]) against
/// the parsed `.gcov` lines for that source file.
pub fn function_coverage(source: &str, gcov_lines: &[LineCoverage]) -> Vec<(String, CoverageSummary)> {
    let funcs = crate::sync::extract_function_implementations(source);
    let mut results = Vec::new();

    for func in funcs {
        let full = format!("{}\n{}", func.signature, func.body);
        let Some(byte_offset) = source.find(&full) else { continue };
        let start_line = source[..byte_offset].matches('\n').count() + 1;
        let end_line = start_line + full.lines().count();

        let summary = summarize(
            &gcov_lines
                .iter()
                .filter(|l| l.line_no >= start_line && l.line_no < end_line)
                .copied()
                .collect::<Vec<_>>(),
        );
        results.push((func.name, summary));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_report() {
        let report = "        -:    0:Source:foo.c\n        1:    1:int main(void) {\n    #####:    2:    return 1;\n        -:    3:}\n";
        let lines = parse_gcov(report);
        assert_eq!(lines.len(), 3);
        let summary = summarize(&lines);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.covered, 1);
    }

    #[test]
    fn percent_with_some_hits() {
        let lines = vec![
            LineCoverage { line_no: 1, count: Some(3) },
            LineCoverage { line_no: 2, count: Some(0) },
            LineCoverage { line_no: 3, count: None },
        ];
        let summary = summarize(&lines);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.covered, 1);
        assert!((summary.percent() - 50.0).abs() < 0.01);
    }
}
