//! High-level project handle and filesystem discovery.
//!
//! A [`Project`] is a validated newc project directory (must contain
//! `src/main.c`, `include/`, and `Makefile`). Use [`Project::discover`] to
//! scan scan directories for all such projects.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::{NewcError, Result};
use crate::module::{list_modules, Module};

/// Which build tool a project's build file targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSystem {
    Make,
    CMake,
}

impl BuildSystem {
    /// Name of the build file this system expects at the project root.
    pub fn build_file_name(self) -> &'static str {
        match self {
            BuildSystem::Make => "Makefile",
            BuildSystem::CMake => "CMakeLists.txt",
        }
    }

    /// Detect the build system in use at `path` by checking which build file exists.
    /// Defaults to `Make` if neither is present (e.g. mid-scaffold).
    pub fn detect(path: &Path) -> Self {
        if path.join("CMakeLists.txt").exists() {
            BuildSystem::CMake
        } else {
            BuildSystem::Make
        }
    }
}

/// An opened newc project with its module list and git state.
#[derive(Debug, Clone)]
pub struct Project {
    /// Project name derived from the root directory's basename.
    pub name: String,
    /// Absolute path to the project root directory.
    pub root: PathBuf,
    pub modules: Vec<Module>,
    /// `true` if `<root>/.git` exists.
    pub has_git: bool,
    /// Build system in use, detected from which build file exists at the root.
    pub build_system: BuildSystem,
}

impl Project {
    /// Open a project at `root`, validating that it is a newc project.
    ///
    /// # Errors
    /// Returns [`NewcError::NotAProject`] if the directory does not contain
    /// `src/main.c`, `include/`, and a `Makefile` or `CMakeLists.txt`.
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
        let build_system = BuildSystem::detect(&root);
        Ok(Self { name, root, modules, has_git, build_system })
    }

    /// Returns `true` if `path` looks like a valid newc project directory.
    pub fn is_newc_project(path: &Path) -> bool {
        path.join("src").join("main.c").exists()
            && path.join("include").is_dir()
            && (path.join("Makefile").exists() || path.join("CMakeLists.txt").exists())
    }

    /// Re-scan `include/` and update the cached module list.
    ///
    /// # Errors
    /// Returns an IO error if the directory cannot be read.
    pub fn refresh_modules(&mut self) -> Result<()> {
        self.modules = list_modules(&self.root)?;
        Ok(())
    }

    /// Recursively discover newc projects under each directory in `scan_dirs`.
    ///
    /// Searches up to 3 directory levels deep. Supports `~` expansion via
    /// [`expand_tilde`]. Results are sorted and deduplicated.
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

/// Expand a leading `~/` to the current user's home directory.
///
/// Returns the path unchanged if it does not start with `~/` or if the
/// home directory cannot be determined.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&s[2..]);
        }
    }
    path.to_path_buf()
}
