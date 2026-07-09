//! Project scaffolding — creates the standard newc directory layout and starter files.

use std::fs;
use std::path::Path;
use std::process::Command;

use chrono::Local;

use crate::config::ProjectConfig;
use crate::error::{NewcError, Result};
use crate::license::{License, spdx_line};
use crate::module::is_valid_c_ident;
use crate::project::BuildSystem;
use crate::templates;

/// Options for creating a new newc project.
#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    /// Project name, used as both the directory name and in generated file headers.
    pub name: String,
    /// If `true`, run `git init` and write `.gitignore` after creating the project.
    pub git_init: bool,
    /// Author name inserted into generated source file comments.
    pub author: String,
    /// Default modules to include in the new project.
    pub modules: Vec<DefaultModule>,
    /// Build file to generate: `Makefile` or `CMakeLists.txt`.
    pub build_system: BuildSystem,
    /// License to apply: writes `LICENSE` and stamps SPDX headers into generated files.
    pub license: Option<License>,
}

/// Built-in module templates that can be included when scaffolding a project.
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultModule {
    Input,
    Math,
    Display,
    Array,
    Strings,
    LinkedList,
    Files,
    TestUtils,
    /// Unity-style test harness (see [`crate::templates::unity_h`]) — an
    /// alternative to `TestUtils` for users who prefer that macro API.
    UnityTest,
}

impl DefaultModule {
    /// Return the four standard default modules: `Input`, `Math`, `Display`, `Array`.
    pub fn all() -> Vec<Self> {
        vec![Self::Input, Self::Math, Self::Display, Self::Array]
    }

    /// Return the filesystem-safe name used for the module's `.c`/`.h` files.
    pub fn name(&self) -> &str {
        match self {
            Self::Input => "input",
            Self::Math => "math",
            Self::Display => "display",
            Self::Array => "array",
            Self::Strings => "strings",
            Self::LinkedList => "linked_list",
            Self::Files => "files",
            Self::TestUtils => "test_utils",
            Self::UnityTest => "unity",
        }
    }
}

/// Create a new newc project directory under `parent`.
///
/// Creates `src/`, `include/`, `build/`, `Makefile`, and `src/main.c`, then
/// writes source files for each module listed in `opts.modules`.
///
/// # Errors
/// Returns [`NewcError::InvalidName`] if the project name is not a valid C identifier,
/// [`NewcError::ProjectExists`] if the directory already exists, or an IO error.
pub fn create_project(opts: &ScaffoldOptions, parent: &Path) -> Result<()> {
    if !is_valid_c_ident(&opts.name) {
        return Err(NewcError::InvalidName(opts.name.clone()));
    }

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

    // Prefixed onto every generated .c/.h file when a license is chosen.
    let spdx = opts.license.map(|l| spdx_line(l.spdx_id()));
    let write_src = |path: std::path::PathBuf, content: String| -> Result<()> {
        let content = match &spdx {
            Some(s) => format!("{s}{content}"),
            None => content,
        };
        fs::write(path, content)?;
        Ok(())
    };

    // main.c
    write_src(
        root.join("src").join("main.c"),
        templates::main_c(author, &date, &include_names),
    )?;

    // Build file — use test variant if test_utils module is included
    let has_tests = opts.modules.contains(&DefaultModule::TestUtils)
        || opts.modules.contains(&DefaultModule::UnityTest);
    match opts.build_system {
        BuildSystem::Make => {
            let makefile = if has_tests {
                templates::MAKEFILE_WITH_TEST
            } else {
                templates::MAKEFILE
            };
            fs::write(root.join("Makefile"), makefile)?;
        }
        BuildSystem::CMake => {
            let cmake = if has_tests {
                templates::CMAKE_LISTS_WITH_TEST
            } else {
                templates::CMAKE_LISTS
            };
            fs::write(root.join("CMakeLists.txt"), cmake)?;
            fs::write(root.join("CMakePresets.json"), templates::CMAKE_PRESETS)?;
        }
    }

    // Default module files
    for module in &opts.modules {
        match module {
            DefaultModule::Input => {
                write_src(
                    root.join("include").join("input.h"),
                    templates::input_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("input.c"),
                    templates::input_c(author, &date),
                )?;
            }
            DefaultModule::Math => {
                write_src(
                    root.join("include").join("math.h"),
                    templates::math_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("math.c"),
                    templates::math_c(author, &date),
                )?;
            }
            DefaultModule::Display => {
                write_src(
                    root.join("include").join("display.h"),
                    templates::display_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("display.c"),
                    templates::display_c(author, &date),
                )?;
            }
            DefaultModule::Array => {
                write_src(
                    root.join("include").join("array.h"),
                    templates::array_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("array.c"),
                    templates::array_c(author, &date),
                )?;
            }
            DefaultModule::Strings => {
                write_src(
                    root.join("include").join("strings.h"),
                    templates::strings_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("strings.c"),
                    templates::strings_c(author, &date),
                )?;
            }
            DefaultModule::LinkedList => {
                write_src(
                    root.join("include").join("linked_list.h"),
                    templates::linked_list_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("linked_list.c"),
                    templates::linked_list_c(author, &date),
                )?;
            }
            DefaultModule::Files => {
                write_src(
                    root.join("include").join("files.h"),
                    templates::files_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("files.c"),
                    templates::files_c(author, &date),
                )?;
            }
            DefaultModule::TestUtils => {
                write_src(
                    root.join("include").join("test_utils.h"),
                    templates::test_utils_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("test_utils.c"),
                    templates::test_utils_c(author, &date),
                )?;
            }
            DefaultModule::UnityTest => {
                write_src(
                    root.join("include").join("unity.h"),
                    templates::unity_h(author, &date),
                )?;
                write_src(
                    root.join("src").join("unity.c"),
                    templates::unity_c(author, &date),
                )?;
            }
        }
    }

    if let Some(license) = opts.license {
        let year = Local::now().format("%Y").to_string();
        fs::write(root.join("LICENSE"), license.license_text(author, &year))?;
        let cfg = ProjectConfig {
            license: Some(license.spdx_id().to_string()),
            ..ProjectConfig::default()
        };
        cfg.save_to(&root)?;
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

/// Detect the current user's name from `git config user.name`, falling back to `"Author"`.
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
            build_system: BuildSystem::Make,
            license: None,
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
            assert!(
                root.join(format!("src/{name}.c")).exists(),
                "{name}.c missing"
            );
            assert!(
                root.join(format!("include/{name}.h")).exists(),
                "{name}.h missing"
            );
        }
    }

    #[test]
    fn creates_cmake_lists_when_requested() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut o = opts("proj", vec![]);
        o.build_system = BuildSystem::CMake;
        create_project(&o, tmp.path()).unwrap();
        let root = tmp.path().join("proj");
        assert!(root.join("CMakeLists.txt").exists());
        assert!(!root.join("Makefile").exists());
    }

    #[test]
    fn license_writes_file_and_stamps_spdx() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut o = opts("proj", vec![DefaultModule::Math]);
        o.license = Some(crate::license::License::Mit);
        create_project(&o, tmp.path()).unwrap();
        let root = tmp.path().join("proj");
        assert!(root.join("LICENSE").exists());
        let main_c = fs::read_to_string(root.join("src/main.c")).unwrap();
        assert!(main_c.starts_with("/* SPDX-License-Identifier: MIT */\n"));
        let math_c = fs::read_to_string(root.join("src/math.c")).unwrap();
        assert!(math_c.starts_with("/* SPDX-License-Identifier: MIT */\n"));
        let cfg = crate::config::ProjectConfig::load_from(&root).unwrap();
        assert_eq!(cfg.license, Some("MIT".to_string()));
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
