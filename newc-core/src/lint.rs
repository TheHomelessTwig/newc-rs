/// Basic static linter for C source files.
/// Text/pattern based — no AST. Catches common beginner mistakes.

/// Lint a `.h` header file. Checks for missing include guard (L010).
pub fn lint_header(content: &str) -> Vec<LintWarning> {
    let mut warnings = Vec::new();
    // L010: missing #ifndef guard — check first 5 non-blank lines for #ifndef or #pragma once
    let has_guard = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(5)
        .any(|l| {
            let t = l.trim();
            t.starts_with("#ifndef") || t.starts_with("#pragma once")
        });
    if !has_guard {
        warnings.push(LintWarning {
            line_no: 1,
            severity: LintSeverity::Warning,
            code: "L010",
            message: "Header file missing include guard (#ifndef HEADER_H / #pragma once)".into(),
        });
    }
    warnings
}

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

        // L001: gets() usage — unsafe buffer overflow (exclude fgets, ungets, etc.)
        if let Some(pos) = t.find("gets(") {
            let prev_is_alpha = pos > 0
                && t[..pos].chars().last().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
            if !prev_is_alpha {
                warnings.push(LintWarning {
                    line_no: lno, severity: LintSeverity::Error, code: "L001",
                    message: "gets() is unsafe — use fgets() instead".into(),
                });
            }
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
        // Matches: printf(variable) or printf(ptr); allows string literals and ALL_CAPS macros
        // Skips fprintf, snprintf, sprintf, dprintf, vprintf, etc. (char before 'printf' is alpha)
        if let Some(pos) = t.find("printf(") {
            let prev_is_alpha = pos > 0
                && t[..pos].chars().last().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
            if !prev_is_alpha {
                let after = t[pos + 7..].trim_start();
                let first_char = after.chars().next().unwrap_or('"');
                let is_safe = first_char == '"'
                    || first_char == ')'
                    || first_char.is_uppercase();
                if !is_safe {
                    warnings.push(LintWarning {
                        line_no: lno, severity: LintSeverity::Warning, code: "L004",
                        message: "printf() called with non-literal format string — possible format attack".into(),
                    });
                }
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
                if lt.starts_with("//") || lt.starts_with('*') { return false; }
                lt.contains("== NULL") || lt.contains("!= NULL")
                    || (lt.starts_with("if") && lt.contains("NULL"))
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

        // L011: free() not followed by ptr = NULL within 3 lines
        if let Some(pos) = t.find("free(") {
            let prev_is_alpha = pos > 0
                && t[..pos].chars().last().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
            if !prev_is_alpha {
                let next3: Vec<&str> = lines.iter().skip(i + 1).take(3).cloned().collect();
                let nulled = next3.iter().any(|l| l.contains("= NULL"));
                if !nulled {
                    warnings.push(LintWarning {
                        line_no: lno, severity: LintSeverity::Info, code: "L011",
                        message: "free() without setting pointer to NULL afterwards — potential use-after-free".into(),
                    });
                }
            }
        }

        // L012: strlen() inside a loop condition
        if (t.starts_with("while") || t.starts_with("for")) && t.contains("strlen(") {
            warnings.push(LintWarning {
                line_no: lno, severity: LintSeverity::Warning, code: "L012",
                message: "strlen() in loop condition — O(n²) performance; cache length in a variable before the loop".into(),
            });
        }

        // L013: atoi()/atof() usage — no error checking possible
        for func in ["atoi(", "atof(", "atol("] {
            if t.contains(func) {
                warnings.push(LintWarning {
                    line_no: lno, severity: LintSeverity::Warning, code: "L013",
                    message: format!("{} has no error reporting — use strtol()/strtod() with endptr check instead", &func[..func.len()-1]),
                });
                break;
            }
        }

        // L014: return &local — returning address of a local variable
        // Heuristic: line contains `return &` and the identifier after & is not `static`/global
        if t.starts_with("return") && t.contains("return &") {
            // Exclude common safe patterns: return &static_var, return &global, return &(*ptr)
            let after = t.trim_start_matches("return").trim().trim_start_matches('&');
            let is_deref = after.starts_with('(');
            if !is_deref {
                warnings.push(LintWarning {
                    line_no: lno, severity: LintSeverity::Error, code: "L014",
                    message: "Returning address of a local variable — undefined behaviour after function returns".into(),
                });
            }
        }

        // L015: comparing a value to a non-zero integer literal that looks like a pointer comparison
        // Heuristic: `== N` or `!= N` where N is a small nonzero int literal (1, -1, 2…)
        // Only flag when it looks like a pointer context (ptr, p_, *name patterns)
        for op in ["== 1", "!= 1", "== -1", "!= -1", "== 2", "!= 2"] {
            if t.contains(op) {
                // Only warn when line also contains * or common pointer naming
                if t.contains('*') || t.contains("ptr") || t.contains("_p ") || t.contains("NULL") {
                    warnings.push(LintWarning {
                        line_no: lno, severity: LintSeverity::Warning, code: "L015",
                        message: format!("Comparing pointer/result with '{op}' — did you mean NULL check (== NULL / != NULL)?"),
                    });
                    break;
                }
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

pub fn has_magic_number(line: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn has(warnings: &[LintWarning], code: &str) -> bool {
        warnings.iter().any(|w| w.code == code)
    }

    // L001 — gets()
    #[test]
    fn l001_gets_triggers() {
        assert!(has(&lint_file("gets(buf);"), "L001"));
    }
    #[test]
    fn l001_in_comment_skipped() {
        assert!(!has(&lint_file("// gets(buf);"), "L001"));
    }
    #[test]
    fn l001_fgets_clean() {
        assert!(!has(&lint_file("fgets(buffer, max_len, stdin);"), "L001"));
    }

    // L002 — strcpy
    #[test]
    fn l002_strcpy_triggers() {
        assert!(has(&lint_file("strcpy(dst, src);"), "L002"));
    }
    #[test]
    fn l002_strncpy_clean() {
        assert!(!has(&lint_file("strncpy(dst, src, 128);"), "L002"));
    }

    // L003 — scanf %s
    #[test]
    fn l003_scanf_unbounded_triggers() {
        assert!(has(&lint_file(r#"scanf("%s", buf);"#), "L003"));
    }
    #[test]
    fn l003_scanf_bounded_clean() {
        assert!(!has(&lint_file(r#"scanf("%127s", buf);"#), "L003"));
    }

    // L004 — printf format string
    #[test]
    fn l004_printf_var_triggers() {
        assert!(has(&lint_file("printf(user_input);"), "L004"));
    }
    #[test]
    fn l004_printf_literal_clean() {
        assert!(!has(&lint_file(r#"printf("hello %s\n", name);"#), "L004"));
    }
    #[test]
    fn l004_printf_uppercase_macro_clean() {
        assert!(!has(&lint_file("printf(MSG_FORMAT, val);"), "L004"));
    }
    #[test]
    fn l004_snprintf_clean() {
        assert!(!has(&lint_file("snprintf(buf, sizeof(buf), \"%s %d\", q, i);"), "L004"));
    }
    #[test]
    fn l004_fprintf_clean() {
        assert!(!has(&lint_file(r#"fprintf(stderr, "Error\n");"#), "L004"));
    }

    // L005 — assignment in condition
    #[test]
    fn l005_assignment_in_if_triggers() {
        assert!(has(&lint_file("if (x = 5) { }"), "L005"));
    }
    #[test]
    fn l005_comparison_clean() {
        assert!(!has(&lint_file("if (x == 5) { }"), "L005"));
    }

    // L006 — sprintf
    #[test]
    fn l006_sprintf_triggers() {
        assert!(has(&lint_file("sprintf(buf, fmt, val);"), "L006"));
    }
    #[test]
    fn l006_snprintf_clean() {
        assert!(!has(&lint_file("snprintf(buf, 128, fmt, val);"), "L006"));
    }

    // L007 — malloc null check
    #[test]
    fn l007_malloc_no_check_triggers() {
        let code = "int *p = malloc(sizeof(int) * n);\n// no check here";
        assert!(has(&lint_file(code), "L007"));
    }
    #[test]
    fn l007_malloc_with_null_check_clean() {
        let code = "int *p = malloc(sizeof(int) * n);\nif (p == NULL) return;";
        assert!(!has(&lint_file(code), "L007"));
    }
    #[test]
    fn l007_comment_null_does_not_suppress() {
        // Bug fix: "// NULL is fine" must not be counted as a null check
        let code = "int *p = malloc(n);\n// NULL is documented elsewhere";
        assert!(has(&lint_file(code), "L007"));
    }

    // L008 — magic numbers
    #[test]
    fn l008_magic_number_triggers() {
        assert!(has(&lint_file("int x = 42;"), "L008"));
    }
    #[test]
    fn l008_common_number_clean() {
        assert!(!has(&lint_file("int x = 0;"), "L008"));
    }

    // L009 — fopen without fclose
    #[test]
    fn l009_fopen_no_close_triggers() {
        assert!(has(&lint_file(r#"FILE *f = fopen("test.txt", "r");"#), "L009"));
    }
    #[test]
    fn l009_fopen_with_close_clean() {
        let code = "FILE *f = fopen(\"test.txt\", \"r\");\nfclose(f);";
        assert!(!has(&lint_file(code), "L009"));
    }

    // Line number accuracy
    #[test]
    fn line_numbers_accurate() {
        let code = "int x = 1;\ngets(buf);\nint y = 2;";
        let warnings = lint_file(code);
        let w = warnings.iter().find(|w| w.code == "L001").unwrap();
        assert_eq!(w.line_no, 2);
    }

    // L010 — missing header guard
    #[test]
    fn l010_missing_guard_triggers() {
        assert!(has(&lint_header("int foo(void);"), "L010"));
    }
    #[test]
    fn l010_ifndef_guard_clean() {
        assert!(!has(&lint_header("#ifndef FOO_H\n#define FOO_H\nint foo(void);\n#endif"), "L010"));
    }
    #[test]
    fn l010_pragma_once_clean() {
        assert!(!has(&lint_header("#pragma once\nint foo(void);"), "L010"));
    }

    // L011 — free without NULL
    #[test]
    fn l011_free_no_null_triggers() {
        assert!(has(&lint_file("free(ptr);\ndo_something();"), "L011"));
    }
    #[test]
    fn l011_free_with_null_clean() {
        assert!(!has(&lint_file("free(ptr);\nptr = NULL;"), "L011"));
    }

    // L012 — strlen in loop condition
    #[test]
    fn l012_strlen_in_while_triggers() {
        assert!(has(&lint_file("while (i < strlen(s)) {"), "L012"));
    }
    #[test]
    fn l012_strlen_outside_loop_clean() {
        assert!(!has(&lint_file("int len = strlen(s);"), "L012"));
    }

    // L013 — atoi/atof usage
    #[test]
    fn l013_atoi_triggers() {
        assert!(has(&lint_file("int n = atoi(argv[1]);"), "L013"));
    }
    #[test]
    fn l013_strtol_clean() {
        assert!(!has(&lint_file("int n = (int)strtol(argv[1], &end, 10);"), "L013"));
    }

    // L014 — return &local
    #[test]
    fn l014_return_address_local_triggers() {
        assert!(has(&lint_file("return &local_var;"), "L014"));
    }
    #[test]
    fn l014_return_deref_clean() {
        assert!(!has(&lint_file("return &(*ptr);"), "L014"));
    }
}
