//! User-defined `main()` templates stored in `~/.config/newc/templates/`.
//!
//! A template captures the full [`MainBuilderState`] (blocks, globals, module
//! includes) so that a common project structure can be reused across new projects.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::main_builder::{GlobalVar, MainBlock, MainBuilderState};

/// A saved `main()` builder configuration that can be re-applied to new projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTemplate {
    /// Display name shown in the template picker.
    pub name: String,
    pub description: String,
    /// Module names to `#include` when the template is applied.
    pub modules: Vec<String>,
    /// Ordered list of statements for the `main()` body.
    pub blocks: Vec<MainBlock>,
    /// File-scope variable declarations.
    pub globals: Vec<GlobalVar>,
}

impl UserTemplate {
    /// Convert this template into a [`MainBuilderState`] ready to load into the builder.
    pub fn to_builder_state(&self) -> MainBuilderState {
        MainBuilderState {
            blocks: self.blocks.clone(),
            globals: self.globals.clone(),
            includes: self.modules.clone(),
            argc_argv: false,
        }
    }
}

/// Persist a template to `~/.config/newc/templates/<safe_name>.toml`.
///
/// The filename is derived from the template name with non-alphanumeric characters
/// replaced by `_`.
///
/// # Errors
/// Returns an error if the directory cannot be created or the file cannot be written.
pub fn save(template: &UserTemplate) -> Result<()> {
    let Some(dir) = template_dir() else {
        return Ok(());
    };
    std::fs::create_dir_all(&dir)?;
    let safe_name: String = template
        .name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let path = dir.join(format!("{safe_name}.toml"));
    let content = toml::to_string_pretty(template)
        .map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Load all templates from `~/.config/newc/templates/`, sorted by filename.
///
/// Returns an empty vec if the directory does not exist or cannot be read.
pub fn load_all() -> Vec<UserTemplate> {
    let Some(dir) = template_dir() else {
        return Vec::new();
    };
    if !dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut templates = Vec::new();
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .map(|e| e.path())
        .collect();
    paths.sort();
    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(t) = toml::from_str::<UserTemplate>(&content)
        {
            templates.push(t);
        }
    }
    templates
}

/// Delete a template by name from `~/.config/newc/templates/`.
///
/// No-ops if the file does not exist.
///
/// # Errors
/// Returns an IO error if the file exists but cannot be removed.
pub fn delete(name: &str) -> Result<()> {
    let Some(dir) = template_dir() else {
        return Ok(());
    };
    let safe_name: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let path = dir.join(format!("{safe_name}.toml"));
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn template_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("templates"))
}
