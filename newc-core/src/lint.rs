/// Basic static linter for C source files.
/// Text/pattern based — no AST. Catches common beginner mistakes.

#[derive(Debug, Clone, PartialEq)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct LintWarning {
    pub line_no: usize,
    pub severity: LintSeverity,
    pub code: &'static str,
    pub message: String,
}

pub fn lint_file(content: &str) -> Vec<LintWarning> {
    let mut warnings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        let lno = i + 1;

        // Skip comments
        if t.starts_with("//") || t.starts_with("*") || t.starts_with("/*") {
            continue;
        }

        // L001: gets() usage — unsafe buffer overflow
        if t.contains("gets(") {
            warnings.push(LintWarning {
                line_no: lno, severity: LintSeverity::Error, code: "L001",
                message: "gets() is unsafe — use fgets() instead".into(),
            });
        }

        // L002: strcpy without bounds check
        if t.contains("strcpy(") && !t.contains("strncpy(") {
            warnings.push(LintWarning {
                line_no: lno, severity: LintSeverity::Warning, code: "L002",
                message: "strcpy() may overflow — consider strncpy() or snprintf()".into(),
            });
        }

        // L003: scanf with unbounded %s
        if t.contains("scanf(") && t.contains("\"%s\"") {
            warnings.push(LintWarning {
                line_no: lno, severity: LintSeverity::Warning, code: "L003",
                message: "scanf(\"%s\") has no width limit — use \"%127s\" or similar".into(),
            });
        }

        // L004: printf with non-literal first arg (possible format string bug)
        // Matches: printf(variable) or printf(ptr)
        if let Some(pos) = t.find("printf(") {
            let after = t[pos + 7..].trim_start();
            if !after.starts_with('"') && !after.starts_with(')') {
                warnings.push(LintWarning {
                    line_no: lno, severity: LintSeverity::Warning, code: "L004",
                    message: "printf() called with non-literal format string — possible format attack".into(),
                });
            }
        }

        // L005: assignment in condition (if (x = y) likely typo)
        if (t.starts_with("if") || t.starts_with("while")) && t.contains('(') {
            if let Some(paren_start) = t.find('(') {
                let inner = &t[paren_start + 1..];
                // Look for single = not preceded by !, <, >, =, not followed by =
                if has_assignment_in_condition(inner) {
                    warnings.push(LintWarning {
                        line_no: lno, severity: LintSeverity::Warning, code: "L005",
                        message: "Possible assignment in condition — did you mean '=='?".into(),
                    });
                }
            }
        }

        // L006: sprintf without bounds
        if t.contains("sprintf(") && !t.contains("snprintf(") {
            warnings.push(LintWarning {
                line_no: lno, severity: LintSeverity::Warning, code: "L006",
                message: "sprintf() may overflow — use snprintf() instead".into(),
            });
        }

        // L007: malloc without NULL check (malloc followed by no null check within 3 lines)
        if t.contains("malloc(") || t.contains("calloc(") || t.contains("realloc(") {
            let next_lines: Vec<&str> = lines.iter().skip(i + 1).take(4).cloned().collect();
            let has_null_check = next_lines.iter().any(|l| {
                let lt = l.trim();
                lt.contains("NULL") || lt.contains("!= NULL") || lt.contains("== NULL")
            });
            if !has_null_check {
                warnings.push(LintWarning {
                    line_no: lno, severity: LintSeverity::Info, code: "L007",
                    message: "malloc/calloc result not checked for NULL".into(),
                });
            }
        }

        // L008: magic numbers (bare integer literals > 9 in expressions, excluding common values)
        if !t.starts_with("#") && !t.starts_with("//") {
            if has_magic_number(t) {
                warnings.push(LintWarning {
                    line_no: lno, severity: LintSeverity::Info, code: "L008",
                    message: "Magic number — consider using a named constant".into(),
                });
            }
        }

        // L009: fopen without fclose check pattern
        if t.contains("fopen(") {
            let rest: Vec<&str> = lines.iter().skip(i + 1).take(20).cloned().collect();
            let has_close = rest.iter().any(|l| l.contains("fclose("));
            if !has_close {
                warnings.push(LintWarning {
                    line_no: lno, severity: LintSeverity::Warning, code: "L009",
                    message: "fopen() with no matching fclose() found in next 20 lines".into(),
                });
            }
        }
    }

    warnings
}

fn has_assignment_in_condition(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut depth = 1i32;
    for i in 0..n {
        match chars[i] {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth <= 0 { break; }
            }
            '=' if depth == 1 => {
                let prev = if i > 0 { chars[i - 1] } else { ' ' };
                let next = if i + 1 < n { chars[i + 1] } else { ' ' };
                // Single = not surrounded by =, !, <, >
                if prev != '=' && prev != '!' && prev != '<' && prev != '>' && next != '=' {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn has_magic_number(line: &str) -> bool {
    // Look for integer literals > 9 that aren't inside array sizes or #define
    // Simple heuristic: digit sequence of 2+ digits not part of identifier
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            let mut num = String::from(c);
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() { num.push(chars.next().unwrap()); }
                else { break; }
            }
            if let Ok(n) = num.parse::<i64>() {
                // Allow 0, 1, -1, 2, 10, 100, 256, 1024 (common powers/constants)
                let common = [0, 1, 2, 10, 100, 256, 512, 1024, 2048, 4096];
                if n > 9 && !common.contains(&n) {
                    return true;
                }
            }
        }
    }
    false
}
