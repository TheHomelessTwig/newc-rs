use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::main_builder::{GlobalVar, MainBlock, MainBuilderState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTemplate {
    pub name: String,
    pub description: String,
    pub modules: Vec<String>,
    pub blocks: Vec<MainBlock>,
    pub globals: Vec<GlobalVar>,
}

impl UserTemplate {
    pub fn to_builder_state(&self) -> MainBuilderState {
        MainBuilderState {
            blocks: self.blocks.clone(),
            globals: self.globals.clone(),
            includes: self.modules.clone(),
            argc_argv: false,
        }
    }
}

pub fn save(template: &UserTemplate) -> Result<()> {
    let Some(dir) = template_dir() else { return Ok(()) };
    std::fs::create_dir_all(&dir)?;
    let safe_name: String = template.name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let path = dir.join(format!("{safe_name}.toml"));
    let content = toml::to_string_pretty(template)
        .map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn load_all() -> Vec<UserTemplate> {
    let Some(dir) = template_dir() else { return Vec::new() };
    if !dir.exists() { return Vec::new(); }
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut templates = Vec::new();
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .map(|e| e.path())
        .collect();
    paths.sort();
    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(t) = toml::from_str::<UserTemplate>(&content) {
                templates.push(t);
            }
        }
    }
    templates
}

pub fn delete(name: &str) -> Result<()> {
    let Some(dir) = template_dir() else { return Ok(()) };
    let safe_name: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let path = dir.join(format!("{safe_name}.toml"));
    if path.exists() { std::fs::remove_file(path)?; }
    Ok(())
}

fn template_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("templates"))
}
