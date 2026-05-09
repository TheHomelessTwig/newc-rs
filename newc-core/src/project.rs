use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::{NewcError, Result};
use crate::module::{list_modules, Module};

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub root: PathBuf,
    pub modules: Vec<Module>,
    pub has_git: bool,
}

impl Project {
    pub fn open(root: PathBuf) -> Result<Self> {
        if !Self::is_newc_project(&root) {
            return Err(NewcError::NotAProject);
        }
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_string());
        let has_git = root.join(".git").exists();
        let modules = list_modules(&root)?;
        Ok(Self { name, root, modules, has_git })
    }

    pub fn is_newc_project(path: &Path) -> bool {
        path.join("src").join("main.c").exists()
            && path.join("include").is_dir()
            && path.join("Makefile").exists()
    }

    pub fn refresh_modules(&mut self) -> Result<()> {
        self.modules = list_modules(&self.root)?;
        Ok(())
    }

    pub fn discover(scan_dirs: &[PathBuf]) -> Vec<PathBuf> {
        let mut found = Vec::new();
        for dir in scan_dirs {
            let expanded = expand_tilde(dir);
            for entry in WalkDir::new(&expanded)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_dir())
            {
                let path = entry.into_path();
                if Self::is_newc_project(&path) {
                    found.push(path);
                }
            }
        }
        found.sort();
        found.dedup();
        found
    }
}

pub fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&s[2..]);
        }
    }
    path.to_path_buf()
}
