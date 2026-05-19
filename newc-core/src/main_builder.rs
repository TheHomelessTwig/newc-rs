//! Visual `main()` builder — structured representation of C `main.c` contents.
//!
//! [`MainBlock`] is the AST-like IR for statements inside `main()`, while
//! [`GlobalVar`] represents file-scope variable declarations. [`MainBuilderState`]
//! holds the full editor state and can round-trip through real `main.c` source.

use serde::{Deserialize, Serialize};

/// A single statement or control-flow construct inside `main()`.
///
/// Each variant maps one-to-one to a C construct and can emit valid C source
/// via [`MainBlock::to_c`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MainBlock {
    /// A local variable declaration, optionally with an initialiser.
    VarDecl {
        type_name: String,
        name: String,
        /// Initialiser expression, or empty string for no initialiser.
        init: String,
        /// `true` when the variable is a C array.
        is_array: bool,
        /// Array dimension expression (e.g. `"10"` or `"MAX"`), empty for scalars.
        array_size: String,
    },
    /// A function call statement, optionally assigned to an lvalue.
    FunctionCall {
        func_name: String,
        /// Positional argument expressions in call order.
        args: Vec<String>,
        /// Left-hand-side variable to assign the return value to, or empty.
        assign_to: String,
        /// Optional inline comment rendered above the call.
        comment: String,
    },
    /// A `/* … */` or `// …` comment.
    Comment(String),
    /// Arbitrary C code that did not match any structured variant during parsing.
    RawCode(String),
    /// An empty line inserted for readability.
    BlankLine,
    /// An `if` / `else` block.
    IfBlock {
        condition: String,
        body: Vec<MainBlock>,
        /// Empty vec when there is no `else` branch.
        else_body: Vec<MainBlock>,
    },
    /// A `while` loop.
    WhileLoop {
        condition: String,
        body: Vec<MainBlock>,
    },
    /// A `for` loop.
    ForLoop {
        /// Initialiser clause (e.g. `"int i = 0"`).
        init: String,
        condition: String,
        /// Increment clause (e.g. `"i++"`).
        increment: String,
        body: Vec<MainBlock>,
    },
}

impl MainBlock {
    /// Short display label for use in GUI list views (e.g. `"Variable"`, `"Call"`).
    pub fn label(&self) -> &str {
        match self {
            MainBlock::VarDecl { .. } => "Variable",
            MainBlock::FunctionCall { .. } => "Call",
            MainBlock::Comment(_) => "Comment",
            MainBlock::RawCode(_) => "Raw",
            MainBlock::BlankLine => "Blank",
            MainBlock::IfBlock { .. } => "If",
            MainBlock::WhileLoop { .. } => "While",
            MainBlock::ForLoop { .. } => "For",
        }
    }

    /// Single-line human-readable summary of the block (e.g. `"int x"`, `"foo(a, b)"`).
    pub fn summary(&self) -> String {
        match self {
            MainBlock::VarDecl { type_name, name, is_array, array_size, .. } => {
                if *is_array {
                    format!("{type_name} {name}[{array_size}]")
                } else {
                    format!("{type_name} {name}")
                }
            }
            MainBlock::FunctionCall { func_name, args, assign_to, .. } => {
                let call = format!("{func_name}({})", args.join(", "));
                if assign_to.is_empty() { call } else { format!("{assign_to} = {call}") }
            }
            MainBlock::Comment(s) => format!("// {}", s.lines().next().unwrap_or("")),
            MainBlock::RawCode(s) => s.lines().next().unwrap_or("").to_string(),
            MainBlock::BlankLine => String::new(),
            MainBlock::IfBlock { condition, .. } => format!("if ({condition})"),
            MainBlock::WhileLoop { condition, .. } => format!("while ({condition})"),
            MainBlock::ForLoop { init, condition, increment, .. } => {
                format!("for ({init}; {condition}; {increment})")
            }
        }
    }

    /// Emit valid C source for this block, indented with a single leading tab.
    pub fn to_c(&self) -> String {
        match self {
            MainBlock::VarDecl { type_name, name, init, is_array, array_size } => {
                if *is_array {
                    if init.is_empty() {
                        format!("\t{type_name} {name}[{array_size}];")
                    } else {
                        format!("\t{type_name} {name}[{array_size}] = {init};")
                    }
                } else if init.is_empty() {
                    format!("\t{type_name} {name};")
                } else {
                    format!("\t{type_name} {name} = {init};")
                }
            }
            MainBlock::FunctionCall { func_name, args, assign_to, comment } => {
                let call = format!("{func_name}({});", args.join(", "));
                let stmt = if assign_to.is_empty() {
                    format!("\t{call}")
                } else {
                    format!("\t{assign_to} = {call}")
                };
                if comment.is_empty() {
                    stmt
                } else {
                    format!("\t/* {comment} */\n{stmt}")
                }
            }
            MainBlock::Comment(s) => {
                s.lines()
                    .map(|l| format!("\t/* {l} */"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            MainBlock::RawCode(s) => s
                .lines()
                .map(|l| format!("\t{l}"))
                .collect::<Vec<_>>()
                .join("\n"),
            MainBlock::BlankLine => String::new(),
            MainBlock::IfBlock { condition, body, else_body } => {
                let body_str = body.iter().map(|b| indent_block(b)).collect::<Vec<_>>().join("\n");
                if else_body.is_empty() {
                    format!("\tif ({condition}) {{\n{body_str}\n\t}}")
                } else {
                    let else_str = else_body.iter().map(|b| indent_block(b)).collect::<Vec<_>>().join("\n");
                    format!("\tif ({condition}) {{\n{body_str}\n\t}} else {{\n{else_str}\n\t}}")
                }
            }
            MainBlock::WhileLoop { condition, body } => {
                let body_str = body.iter().map(|b| indent_block(b)).collect::<Vec<_>>().join("\n");
                format!("\twhile ({condition}) {{\n{body_str}\n\t}}")
            }
            MainBlock::ForLoop { init, condition, increment, body } => {
                let body_str = body.iter().map(|b| indent_block(b)).collect::<Vec<_>>().join("\n");
                format!("\tfor ({init}; {condition}; {increment}) {{\n{body_str}\n\t}}")
            }
        }
    }
}

/// Add one extra tab level to all lines of a block's C output.
fn indent_block(block: &MainBlock) -> String {
    block.to_c()
        .lines()
        .map(|l| format!("\t{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A file-scope variable declaration placed before `main()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GlobalVar {
    pub type_name: String,
    pub name: String,
    /// Initialiser expression, or empty string for no initialiser.
    pub init: String,
    /// `true` when the variable is a C array.
    pub is_array: bool,
    /// Array dimension expression, empty for scalars.
    pub array_size: String,
    /// `true` to emit a `static` storage-class specifier.
    pub is_static: bool,
}

impl GlobalVar {
    /// Emit a complete C variable declaration line (no trailing newline).
    pub fn to_c(&self) -> String {
        let prefix = if self.is_static { "static " } else { "" };
        if self.is_array {
            if self.init.is_empty() {
                format!("{prefix}{} {}[{}];", self.type_name, self.name, self.array_size)
            } else {
                format!("{prefix}{} {}[{}] = {};", self.type_name, self.name, self.array_size, self.init)
            }
        } else if self.init.is_empty() {
            format!("{prefix}{} {};", self.type_name, self.name)
        } else {
            format!("{prefix}{} {} = {};", self.type_name, self.name, self.init)
        }
    }
}

/// A parsed function parameter (type + name).
#[derive(Debug, Clone)]
pub struct FuncParam {
    pub type_name: String,
    pub name: String,
}

/// Parse the parameter list from a C function signature string.
///
/// Returns an empty vec if the signature contains no `(…)` or only `void`.
pub fn parse_params(signature: &str) -> Vec<FuncParam> {
    let start = match signature.find('(') { Some(i) => i, None => return Vec::new() };
    let end   = match signature.rfind(')') { Some(i) => i, None => return Vec::new() };
    let inner = &signature[start + 1..end];
    split_args(inner)
        .into_iter()
        .filter_map(|p| parse_single_param(&p))
        .collect()
}

fn parse_single_param(p: &str) -> Option<FuncParam> {
    let p = p.trim();
    if p.is_empty() || p == "void" { return None; }

    // Handle trailing [] for array params: "int arr[]" or "int arr[10]"
    let (p_no_bracket, bracket_suffix) = if let Some(b) = p.find('[') {
        let suffix = &p[b..]; // e.g. "[]" or "[10]"
        (&p[..b], suffix)
    } else {
        (p, "")
    };

    // Last whitespace-or-star-bounded token is the name
    let trimmed = p_no_bracket.trim_end();
    let last_sep = trimmed.rfind(|c: char| c == ' ' || c == '*')?;
    let raw_name = trimmed[last_sep + 1..].trim();
    let type_part = trimmed[..last_sep + 1].trim();

    let name = format!("{}{}", raw_name, bracket_suffix);
    if raw_name.is_empty() { return None; }

    Some(FuncParam {
        type_name: type_part.to_string(),
        name,
    })
}

/// Generate a complete `main.c` source file from structured builder state.
///
/// # Arguments
/// * `blocks` — ordered list of statements for the body of `main()`.
/// * `globals` — file-scope variable declarations inserted before `main()`.
/// * `author` / `date` — substituted into the file-header comment.
/// * `includes` — module names whose headers are `#include`d (without `.h`).
/// * `argc_argv` — if `true`, `main` is declared with `int argc, char *argv[]`.
///
/// # Returns
/// A complete, syntactically valid `main.c` as a `String`.
pub fn generate_main_c(
    blocks: &[MainBlock],
    globals: &[GlobalVar],
    author: &str,
    date: &str,
    includes: &[String],
    argc_argv: bool,
) -> String {
    let include_lines: String = {
        let mut lines = vec!["#include <stdio.h>".to_string()];
        if argc_argv {
            lines.push("#include <stdlib.h>".to_string());
        }
        for m in includes {
            lines.push(format!("#include \"{m}.h\""));
        }
        lines.join("\n")
    };

    let global_section = if globals.is_empty() {
        String::new()
    } else {
        let g = globals.iter().map(|g| g.to_c()).collect::<Vec<_>>().join("\n");
        format!("\n/* Global variables */\n{g}\n")
    };

    let body = blocks
        .iter()
        .map(|b| b.to_c())
        .collect::<Vec<_>>()
        .join("\n");

    let (sig, ret) = if argc_argv {
        ("int main(int argc, char *argv[])", "\treturn EXIT_SUCCESS;")
    } else {
        ("int main(void)", "\treturn 0;")
    };

    format!(
        "/*\n * Author: {author}\n * Purpose: ...\n *\n * Date: {date}\n */\n\n/* Header files */\n{include_lines}\n{global_section}\n{sig}\n{{\n{body}\n{ret}\n}}\n"
    )
}

/// Full editor state for the visual `main()` builder.
///
/// Can be serialised, loaded from an existing `main.c`, and rendered back
/// to C source via [`MainBuilderState::preview`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MainBuilderState {
    pub blocks: Vec<MainBlock>,
    pub globals: Vec<GlobalVar>,
    /// Module names to `#include` (without `.h` extension).
    pub includes: Vec<String>,
    /// When `true`, `main` is declared with `argc` and `argv` parameters.
    pub argc_argv: bool,
}

impl MainBuilderState {
    /// Create an empty state pre-populated with the module headers found in `<root>/include/`.
    pub fn from_project(root: &std::path::Path) -> Self {
        let includes = detect_includes(root);
        Self { blocks: Vec::new(), globals: Vec::new(), includes, argc_argv: false }
    }

    /// Load from existing src/main.c — parses globals and body into state.
    pub fn load_from_main_c(root: &std::path::Path) -> Self {
        let includes = detect_includes(root);
        let main_c = root.join("src").join("main.c");
        let Ok(source) = std::fs::read_to_string(&main_c) else {
            return Self { blocks: Vec::new(), globals: Vec::new(), includes, argc_argv: false };
        };
        let blocks = parse_main_body(&source);
        let globals = parse_globals(&source);
        let argc_argv = detect_argc_argv(&source);
        Self { blocks, globals, includes, argc_argv }
    }

    /// Render the current state as a complete `main.c` source string.
    pub fn preview(&self, author: &str, date: &str) -> String {
        generate_main_c(&self.blocks, &self.globals, author, date, &self.includes, self.argc_argv)
    }
}

/// Parse global variable declarations from a main.c — lines between includes and main().
fn parse_globals(source: &str) -> Vec<GlobalVar> {
    let mut globals = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        // Stop at main() definition
        if t.contains("main(") { break; }
        // Skip includes, comments, blank lines, preprocessor
        if t.is_empty() || t.starts_with('#') || t.starts_with("/*")
            || t.starts_with("*") || t.starts_with("//") { continue; }

        let is_static = t.starts_with("static ");
        let t_no_static = if is_static { t.trim_start_matches("static").trim() } else { t };

        if let Some(vd) = try_parse_var_decl(t_no_static) {
            // Convert VarDecl block to GlobalVar
            if let MainBlock::VarDecl { type_name, name, init, is_array, array_size } = vd {
                globals.push(GlobalVar { type_name, name, init, is_array, array_size, is_static });
            }
        }
    }
    globals
}

fn detect_includes(root: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.join("include"))
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("h"))
        .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    v.sort();
    v
}

fn detect_argc_argv(source: &str) -> bool {
    source.lines().any(|l| {
        let t = l.trim();
        t.contains("main(") && t.contains("argc")
    })
}

/// Best-effort parser: converts main() body lines into MainBlocks.
fn parse_main_body(source: &str) -> Vec<MainBlock> {
    // Find main() body — everything between the outer { } of main
    let lines: Vec<&str> = source.lines().collect();
    let mut in_main = false;
    let mut brace_depth: i32 = 0;
    let mut body_lines: Vec<&str> = Vec::new();

    for line in &lines {
        let t = line.trim();
        if !in_main {
            if t.contains("main(") && (t.starts_with("int") || t.ends_with('{')) {
                in_main = true;
            }
            if in_main && t.contains('{') {
                brace_depth += t.chars().filter(|&c| c == '{').count() as i32;
                brace_depth -= t.chars().filter(|&c| c == '}').count() as i32;
            }
            continue;
        }
        let prev_depth = brace_depth;
        brace_depth += t.chars().filter(|&c| c == '{').count() as i32;
        brace_depth -= t.chars().filter(|&c| c == '}').count() as i32;
        if brace_depth <= 0 {
            break; // closing } of main
        }
        // Skip the Allman opening brace (first line after signature where depth goes 0→1)
        if prev_depth == 0 {
            continue;
        }
        body_lines.push(line);
    }

    let mut blocks = Vec::new();
    let mut i = 0;
    let mut body_depth: i32 = 0; // nesting within body (0 = top level of main)
    while i < body_lines.len() {
        let raw = body_lines[i];
        let t = raw.trim();
        i += 1;

        let opens = t.chars().filter(|&c| c == '{').count() as i32;
        let closes = t.chars().filter(|&c| c == '}').count() as i32;

        if t.is_empty() {
            body_depth += opens - closes;
            blocks.push(MainBlock::BlankLine);
            continue;
        }
        // Only skip the auto-generated top-level return
        if t.starts_with("return") && body_depth == 0 {
            body_depth += opens - closes;
            continue;
        }
        body_depth += opens - closes;
        // Block comment /* ... */
        if t.starts_with("/*") {
            let mut comment = t.trim_start_matches("/*").trim_end_matches("*/").trim().to_string();
            // Multi-line comment
            if !t.contains("*/") {
                loop {
                    if i >= body_lines.len() { break; }
                    let next = body_lines[i].trim();
                    i += 1;
                    if next.contains("*/") {
                        let part = next.trim_end_matches("*/").trim_start_matches('*').trim();
                        if !part.is_empty() { comment.push(' '); comment.push_str(part); }
                        break;
                    }
                    let part = next.trim_start_matches('*').trim();
                    if !part.is_empty() { comment.push(' '); comment.push_str(part); }
                }
            }
            blocks.push(MainBlock::Comment(comment));
            continue;
        }
        // Line comment
        if t.starts_with("//") {
            blocks.push(MainBlock::Comment(t.trim_start_matches("//").trim().to_string()));
            continue;
        }
        // Variable declaration: starts with a type keyword + identifier + optional [size] + optional = init + ;
        if let Some(vd) = try_parse_var_decl(t) {
            blocks.push(vd);
            continue;
        }
        // Function call: identifier(...);  or  lvalue = identifier(...);
        if let Some(fc) = try_parse_func_call(t) {
            blocks.push(fc);
            continue;
        }
        // Fallback
        blocks.push(MainBlock::RawCode(t.to_string()));
    }

    blocks
}

fn try_parse_var_decl(t: &str) -> Option<MainBlock> {
    if !t.ends_with(';') { return None; }
    let t = t.trim_end_matches(';').trim();

    // Must start with a known type word and not contain '('  (to exclude calls)
    static TYPE_WORDS: &[&str] = &[
        "int", "double", "float", "char", "long", "short", "unsigned", "signed",
        "size_t", "bool", "void",
    ];
    let starts_with_type = TYPE_WORDS.iter().any(|kw| {
        t.starts_with(kw) && t.len() > kw.len()
            && !t.chars().nth(kw.len()).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false)
    });
    if !starts_with_type { return None; }
    if t.contains('(') { return None; } // function pointer or call

    // Split on '=' for init
    let (decl_part, init) = if let Some(eq) = t.find('=') {
        (t[..eq].trim(), t[eq + 1..].trim().to_string())
    } else {
        (t, String::new())
    };

    // Parse: type name[size]  or  type *name  or  type name
    let tokens: Vec<&str> = decl_part.split_whitespace().collect();
    if tokens.len() < 2 { return None; }

    let type_name = tokens[..tokens.len() - 1].join(" ");
    let name_part = tokens.last()?;

    let (name, is_array, array_size) = if let Some(bracket) = name_part.find('[') {
        let name = name_part[..bracket].trim_start_matches('*').to_string();
        let size = name_part[bracket + 1..].trim_end_matches(']').trim().to_string();
        (name, true, size)
    } else {
        (name_part.trim_start_matches('*').to_string(), false, String::new())
    };

    if name.is_empty() { return None; }

    Some(MainBlock::VarDecl {
        type_name: type_name.trim_start_matches('*').to_string(),
        name,
        init,
        is_array,
        array_size,
    })
}

fn try_parse_func_call(t: &str) -> Option<MainBlock> {
    if !t.ends_with(';') { return None; }
    let t = t.trim_end_matches(';').trim();

    // Optional: "assign_to = func(...)"
    let (assign_to, rest) = if let Some(eq) = find_assign_eq(t) {
        (t[..eq].trim().to_string(), t[eq + 1..].trim())
    } else {
        (String::new(), t)
    };

    // rest must be "funcname(...)"
    let paren = rest.find('(')?;
    let func_name = rest[..paren].trim();
    if func_name.is_empty() || !func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    if !rest.ends_with(')') { return None; }
    let args_str = &rest[paren + 1..rest.len() - 1];
    let args = split_args(args_str);

    Some(MainBlock::FunctionCall {
        func_name: func_name.to_string(),
        args,
        assign_to,
        comment: String::new(),
    })
}

/// Find '=' that is an assignment (not '==', '<=', '>=', '!=').
fn find_assign_eq(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '=' {
            let prev = if i > 0 { chars[i - 1] } else { ' ' };
            let next = chars.get(i + 1).copied().unwrap_or(' ');
            if prev != '!' && prev != '<' && prev != '>' && prev != '=' && next != '=' {
                return Some(i);
            }
        }
    }
    None
}

fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    let mut in_str = false;

    for c in s.chars() {
        match c {
            '"' => { in_str = !in_str; current.push(c); }
            '(' | '[' | '{' if !in_str => { depth += 1; current.push(c); }
            ')' | ']' | '}' if !in_str => { depth -= 1; current.push(c); }
            ',' if !in_str && depth == 0 => {
                let t = current.trim().to_string();
                if !t.is_empty() { args.push(t); }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let t = current.trim().to_string();
    if !t.is_empty() { args.push(t); }
    args
}
