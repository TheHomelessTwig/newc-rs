//! Header file editor — reading and writing the `SYNC_IGNORE` block and generating
//! C declaration snippets.
//!
//! The `SYNC_IGNORE` block (bounded by `/* SYNC_IGNORE_START */` / `/* SYNC_IGNORE_END */`)
//! preserves user-written content (structs, enums, `#define`s) across header regeneration
//! by [`crate::sync::sync_module`].

use std::path::Path;

use crate::error::Result;

/// Reads and writes the SYNC_IGNORE block in a header file.
/// Everything between /* SYNC_IGNORE_START */ and /* SYNC_IGNORE_END */ is
/// treated as free-form user content.
pub fn read_ignore_block(hdr: &Path) -> Result<String> {
    let content = std::fs::read_to_string(hdr)?;
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
    Ok(lines.join("\n"))
}

/// Writes a new SYNC_IGNORE block into the header, replacing whatever was there.
/// If no SYNC_IGNORE markers exist yet, injects them after `#define GUARD_H`.
pub fn write_ignore_block(hdr: &Path, new_content: &str) -> Result<()> {
    let raw = std::fs::read_to_string(hdr)?;
    let new_block = format!(
        "/* SYNC_IGNORE_START */\n{}\n/* SYNC_IGNORE_END */",
        new_content.trim_end()
    );

    let result = if raw.contains("SYNC_IGNORE_START") {
        // Replace existing block
        let mut out = String::new();
        let mut skip = false;
        for line in raw.lines() {
            if line.contains("SYNC_IGNORE_START") {
                out.push_str(&new_block);
                out.push('\n');
                skip = true;
                continue;
            }
            if line.contains("SYNC_IGNORE_END") {
                skip = false;
                continue;
            }
            if !skip {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    } else {
        // Inject after #define GUARD_H line
        let mut out = String::new();
        let mut injected = false;
        for line in raw.lines() {
            out.push_str(line);
            out.push('\n');
            if !injected && line.trim().starts_with("#define") && line.contains("_H") {
                out.push('\n');
                out.push_str(&new_block);
                out.push('\n');
                injected = true;
            }
        }
        if !injected {
            out.push_str(&new_block);
            out.push('\n');
        }
        out
    };

    std::fs::write(hdr, result)?;
    Ok(())
}

// Section templates the GUI can insert into the `SYNC_IGNORE` block.

/// Return a `typedef struct` skeleton with a placeholder field.
pub fn struct_template(name: &str) -> String {
    format!(
        "typedef struct {{\n\t/* TODO: add fields */\n\tint placeholder;\n}} {name};\n"
    )
}

/// Return a `typedef enum` skeleton with three sentinel values.
pub fn enum_template(name: &str) -> String {
    format!(
        "typedef enum {{\n\t{name}_FIRST = 0,\n\t{name}_SECOND,\n\t{name}_COUNT\n}} {name};\n"
    )
}

/// Return a `#define` preprocessor macro line.
pub fn define_template(name: &str, value: &str) -> String {
    format!("#define {name} {value}\n")
}

/// Return a `typedef` alias declaration.
pub fn typedef_template(original: &str, alias: &str) -> String {
    format!("typedef {original} {alias};\n")
}

/// Return a `static const` typed constant declaration.
pub fn constant_template(type_name: &str, name: &str, value: &str) -> String {
    format!("static const {type_name} {name} = {value};\n")
}
