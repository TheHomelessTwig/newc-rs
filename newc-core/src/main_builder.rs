use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MainBlock {
    VarDecl {
        type_name: String,
        name: String,
        init: String,
        is_array: bool,
        array_size: String,
    },
    FunctionCall {
        func_name: String,
        args: Vec<String>,
        assign_to: String,
        comment: String,
    },
    Comment(String),
    RawCode(String),
    BlankLine,
}

impl MainBlock {
    pub fn label(&self) -> &str {
        match self {
            MainBlock::VarDecl { .. } => "Variable",
            MainBlock::FunctionCall { .. } => "Call",
            MainBlock::Comment(_) => "Comment",
            MainBlock::RawCode(_) => "Raw",
            MainBlock::BlankLine => "Blank",
        }
    }

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
                if assign_to.is_empty() {
                    call
                } else {
                    format!("{assign_to} = {call}")
                }
            }
            MainBlock::Comment(s) => format!("// {}", s.lines().next().unwrap_or("")),
            MainBlock::RawCode(s) => s.lines().next().unwrap_or("").to_string(),
            MainBlock::BlankLine => String::new(),
        }
    }

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
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GlobalVar {
    pub type_name: String,
    pub name: String,
    pub init: String,
    pub is_array: bool,
    pub array_size: String,
    pub is_static: bool,
}

impl GlobalVar {
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

/// Parse parameter list from a C function signature string.
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

/// Generate a complete main.c from blocks, globals, and includes.
pub fn generate_main_c(
    blocks: &[MainBlock],
    globals: &[GlobalVar],
    author: &str,
    date: &str,
    includes: &[String],
) -> String {
    let include_lines: String = {
        let mut lines = vec!["#include <stdio.h>".to_string()];
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

    format!(
        "/*\n * Author: {author}\n * Purpose: ...\n *\n * Date: {date}\n */\n\n/* Header files */\n{include_lines}\n{global_section}\nint main(void)\n{{\n{body}\n\treturn 0;\n}}\n"
    )
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MainBuilderState {
    pub blocks: Vec<MainBlock>,
    pub globals: Vec<GlobalVar>,
    pub includes: Vec<String>,
}

impl MainBuilderState {
    pub fn from_project(root: &std::path::Path) -> Self {
        let includes = detect_includes(root);
        Self { blocks: Vec::new(), globals: Vec::new(), includes }
    }

    /// Load from existing src/main.c — parses globals and body into state.
    pub fn load_from_main_c(root: &std::path::Path) -> Self {
        let includes = detect_includes(root);
        let main_c = root.join("src").join("main.c");
        let (blocks, globals) = std::fs::read_to_string(&main_c)
            .map(|s| (parse_main_body(&s), parse_globals(&s)))
            .unwrap_or_default();
        Self { blocks, globals, includes }
    }

    pub fn preview(&self, author: &str, date: &str) -> String {
        generate_main_c(&self.blocks, &self.globals, author, date, &self.includes)
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
            // Match: "int main(" or "int main(void)"
            if t.contains("main(") && (t.starts_with("int") || t.ends_with('{')) {
                in_main = true;
            }
            if in_main && t.contains('{') {
                brace_depth += t.chars().filter(|&c| c == '{').count() as i32;
                brace_depth -= t.chars().filter(|&c| c == '}').count() as i32;
            }
            continue;
        }
        brace_depth += t.chars().filter(|&c| c == '{').count() as i32;
        brace_depth -= t.chars().filter(|&c| c == '}').count() as i32;
        if brace_depth <= 0 {
            break; // closing } of main
        }
        body_lines.push(line);
    }

    let mut blocks = Vec::new();
    let mut i = 0;
    while i < body_lines.len() {
        let raw = body_lines[i];
        let t = raw.trim();
        i += 1;

        if t.is_empty() {
            blocks.push(MainBlock::BlankLine);
            continue;
        }
        if t.starts_with("return") {
            continue; // auto-generated
        }
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
