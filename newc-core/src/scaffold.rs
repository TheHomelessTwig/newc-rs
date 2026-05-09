use std::fs;
use std::path::Path;
use std::process::Command;

use chrono::Local;

use crate::error::{NewcError, Result};
use crate::templates;

#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    pub name: String,
    pub git_init: bool,
    pub author: String,
    pub modules: Vec<DefaultModule>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultModule {
    Input,
    Math,
    Display,
    Array,
}

impl DefaultModule {
    pub fn all() -> Vec<Self> {
        vec![Self::Input, Self::Math, Self::Display, Self::Array]
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Input => "input",
            Self::Math => "math",
            Self::Display => "display",
            Self::Array => "array",
        }
    }
}

pub fn create_project(opts: &ScaffoldOptions, parent: &Path) -> Result<()> {
    let root = parent.join(&opts.name);

    if root.exists() {
        return Err(NewcError::ProjectExists(opts.name.clone()));
    }

    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("include"))?;
    fs::create_dir_all(root.join("build"))?;

    let date = Local::now().format("%d/%m/%Y").to_string();
    let author = &opts.author;

    // Determine which default module names are included (for main.c #includes)
    let include_names: Vec<&str> = opts.modules.iter().map(|m| m.name()).collect();

    // main.c
    fs::write(
        root.join("src").join("main.c"),
        templates::main_c(author, &date, &include_names),
    )?;

    // Makefile
    fs::write(root.join("Makefile"), templates::MAKEFILE)?;

    // Default module files
    for module in &opts.modules {
        match module {
            DefaultModule::Input => {
                fs::write(root.join("include").join("input.h"), templates::input_h(author, &date))?;
                fs::write(root.join("src").join("input.c"), templates::input_c(author, &date))?;
            }
            DefaultModule::Math => {
                fs::write(root.join("include").join("math.h"), templates::math_h(author, &date))?;
                fs::write(root.join("src").join("math.c"), templates::math_c(author, &date))?;
            }
            DefaultModule::Display => {
                fs::write(
                    root.join("include").join("display.h"),
                    templates::display_h(author, &date),
                )?;
                fs::write(
                    root.join("src").join("display.c"),
                    templates::display_c(author, &date),
                )?;
            }
            DefaultModule::Array => {
                fs::write(root.join("include").join("array.h"), templates::array_h(author, &date))?;
                fs::write(root.join("src").join("array.c"), templates::array_c(author, &date))?;
            }
        }
    }

    if opts.git_init {
        fs::write(root.join(".gitignore"), templates::GITIGNORE)?;
        Command::new("git")
            .arg("init")
            .current_dir(&root)
            .output()
            .ok();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn opts(name: &str, modules: Vec<DefaultModule>) -> ScaffoldOptions {
        ScaffoldOptions {
            name: name.to_string(),
            git_init: false,
            author: "Test".to_string(),
            modules,
        }
    }

    #[test]
    fn creates_standard_dirs_and_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_project(&opts("proj", vec![]), tmp.path()).unwrap();
        let root = tmp.path().join("proj");
        assert!(root.join("src").is_dir());
        assert!(root.join("include").is_dir());
        assert!(root.join("build").is_dir());
        assert!(root.join("Makefile").exists());
        assert!(root.join("src/main.c").exists());
    }

    #[test]
    fn creates_all_default_module_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_project(&opts("proj", DefaultModule::all()), tmp.path()).unwrap();
        let root = tmp.path().join("proj");
        for name in ["input", "math", "display", "array"] {
            assert!(root.join(format!("src/{name}.c")).exists(), "{name}.c missing");
            assert!(root.join(format!("include/{name}.h")).exists(), "{name}.h missing");
        }
    }

    #[test]
    fn duplicate_project_returns_error() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_project(&opts("proj", vec![]), tmp.path()).unwrap();
        assert!(create_project(&opts("proj", vec![]), tmp.path()).is_err());
    }

    #[test]
    fn main_c_contains_module_includes() {
        let tmp = tempfile::TempDir::new().unwrap();
        create_project(&opts("proj", DefaultModule::all()), tmp.path()).unwrap();
        let main_c = fs::read_to_string(tmp.path().join("proj/src/main.c")).unwrap();
        assert!(main_c.contains("#include \"input.h\""));
        assert!(main_c.contains("#include \"math.h\""));
        assert!(main_c.contains("#include \"display.h\""));
        assert!(main_c.contains("#include \"array.h\""));
    }
}

pub fn detect_author() -> String {
    Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Author".to_string())
}
