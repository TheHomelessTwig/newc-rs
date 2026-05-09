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

/// Generate a complete main.c body from a list of blocks.
/// `includes` = module names to #include (e.g. ["input", "array"]).
pub fn generate_main_c(
    blocks: &[MainBlock],
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

    let body = blocks
        .iter()
        .map(|b| b.to_c())
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "/*\n * Author: {author}\n * Purpose: ...\n *\n * Date: {date}\n */\n\n/* Header files */\n{include_lines}\n\nint main(void)\n{{\n{body}\n\treturn 0;\n}}\n"
    )
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MainBuilderState {
    pub blocks: Vec<MainBlock>,
    pub includes: Vec<String>,
}

impl MainBuilderState {
    pub fn from_project(root: &std::path::Path) -> Self {
        // Auto-detect modules from include/*.h
        let includes: Vec<String> = std::fs::read_dir(root.join("include"))
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("h"))
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .collect();

        Self { blocks: Vec::new(), includes }
    }

    pub fn preview(&self, author: &str, date: &str) -> String {
        generate_main_c(&self.blocks, author, date, &self.includes)
    }
}
