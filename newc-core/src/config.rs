//! Global application configuration and per-project configuration overrides.
//!
//! The global config is stored at `~/.config/newc/config.toml` (TOML) and is
//! loaded once at startup. Per-project overrides live in `<project>/.newc_config.toml`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::project::expand_tilde;

/// A named group of project paths shown together in the GUI library view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub name: String,
    /// Absolute paths of projects belonging to this workspace.
    #[serde(default)]
    pub paths: Vec<PathBuf>,
}

/// Global application configuration persisted to `~/.config/newc/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Terminal emulator command used by [`AppConfig::open_in_editor`].
    #[serde(default = "default_terminal")]
    pub terminal: String,
    /// Editor command launched inside the terminal.
    #[serde(default = "default_editor")]
    pub editor: String,
    /// Directories searched when discovering projects (supports `~` expansion).
    #[serde(default = "default_scan_dirs")]
    pub scan_dirs: Vec<String>,
    /// UI colour theme: `"dark"` or `"light"`.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Named groups of projects; the special `"__archived__"` group holds hidden projects.
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
    /// Style argument passed to `clang-format --style=`.
    #[serde(default = "default_clang_format_style")]
    pub clang_format_style: String,
    /// Monospace font size (logical pixels) used by all code views and editors.
    #[serde(default = "default_code_font_size")]
    pub code_font_size: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            terminal: default_terminal(),
            editor: default_editor(),
            scan_dirs: default_scan_dirs(),
            theme: default_theme(),
            workspaces: Vec::new(),
            clang_format_style: default_clang_format_style(),
            code_font_size: default_code_font_size(),
        }
    }
}

fn default_clang_format_style() -> String {
    "file".to_string()
}

fn default_code_font_size() -> f32 {
    13.0
}

fn default_terminal() -> String {
    // Detect a reasonable default for the current platform
    #[cfg(target_os = "macos")]
    return "Terminal".to_string();
    #[cfg(target_os = "windows")]
    return "wt".to_string();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Try to detect common Linux terminals
        for term in ["foot", "kitty", "alacritty", "wezterm", "xterm"] {
            if which(term) {
                return term.to_string();
            }
        }
        "xterm".to_string()
    }
}

fn default_editor() -> String {
    for ed in ["nvim", "vim", "nano", "code"] {
        if which(ed) {
            return ed.to_string();
        }
    }
    "nvim".to_string()
}

fn default_scan_dirs() -> Vec<String> {
    vec!["~/projects".to_string()]
}

fn default_theme() -> String {
    "dark".to_string()
}

fn which(bin: &str) -> bool {
    // `which` on Unix/macOS, `where` on Windows
    #[cfg(target_os = "windows")]
    let cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let cmd = "which";

    std::process::Command::new(cmd)
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// A named set of extra compiler flags selectable in the build panel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildProfile {
    pub name: String,
    pub cflags: String,
}

/// Per-project setting overrides stored in `<project>/.newc_config.toml`.
/// Any `Some` value overrides the corresponding global `AppConfig` field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub editor: Option<String>,
    pub terminal: Option<String>,
    pub clang_format_style: Option<String>,
    /// Named build profiles (extra CFLAGS) selectable in the build panel.
    #[serde(default)]
    pub build_profiles: Vec<BuildProfile>,
}

impl ProjectConfig {
    /// Load per-project overrides from `<root>/.newc_config.toml`.
    ///
    /// Returns `None` if the file does not exist or cannot be parsed.
    pub fn load_from(root: &std::path::Path) -> Option<Self> {
        let path = root.join(".newc_config.toml");
        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Write per-project overrides to `<root>/.newc_config.toml`.
    ///
    /// # Errors
    /// Returns an error if serialisation or the file write fails.
    pub fn save_to(&self, root: &std::path::Path) -> crate::error::Result<()> {
        let path = root.join(".newc_config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl AppConfig {
    /// Load the global config from disk, falling back to defaults on any error.
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        if !path.exists() {
            return Self::default();
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&content).unwrap_or_default()
    }

    /// Persist the current config to disk, creating parent directories as needed.
    ///
    /// # Errors
    /// Returns an error if serialisation or the write fails.
    pub fn save(&self) -> crate::error::Result<()> {
        let Some(path) = config_path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Return [`scan_dirs`](Self::scan_dirs) as expanded `PathBuf`s with `~` resolved.
    pub fn scan_paths(&self) -> Vec<PathBuf> {
        self.scan_dirs
            .iter()
            .map(|s| expand_tilde(&PathBuf::from(s)))
            .collect()
    }

    /// Returns `true` unless the theme is set to `"light"`.
    pub fn is_dark(&self) -> bool {
        self.theme != "light"
    }

    /// Create a new empty workspace with the given name.
    ///
    /// Returns `false` if a workspace with that name already exists.
    pub fn add_workspace(&mut self, name: impl Into<String>) -> bool {
        let name = name.into();
        if self.workspaces.iter().any(|w| w.name == name) {
            return false;
        }
        self.workspaces.push(Workspace { name, paths: Vec::new() });
        true
    }

    /// Add `path` to the named workspace, creating the workspace if it does not exist.
    /// Duplicate paths are silently ignored.
    pub fn add_to_workspace(&mut self, name: &str, path: PathBuf) {
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.name == name) {
            if !ws.paths.contains(&path) {
                ws.paths.push(path);
            }
        } else {
            self.workspaces.push(Workspace { name: name.to_string(), paths: vec![path] });
        }
    }

    /// Remove `path` from every workspace it appears in.
    pub fn remove_from_all_workspaces(&mut self, path: &PathBuf) {
        for ws in &mut self.workspaces {
            ws.paths.retain(|p| p != path);
        }
    }

    /// Move `path` to the hidden `"__archived__"` workspace, removing it from all others.
    pub fn archive(&mut self, path: PathBuf) {
        self.remove_from_all_workspaces(&path);
        self.add_to_workspace("__archived__", path);
    }

    /// Remove `path` from the `"__archived__"` workspace.
    pub fn unarchive(&mut self, path: &PathBuf) {
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.name == "__archived__") {
            ws.paths.retain(|p| p != path);
        }
    }

    /// Returns `true` if `path` is in the `"__archived__"` workspace.
    pub fn is_archived(&self, path: &PathBuf) -> bool {
        self.workspaces
            .iter()
            .find(|w| w.name == "__archived__")
            .map(|w| w.paths.contains(path))
            .unwrap_or(false)
    }

    /// Launch the project root in the user's configured editor+terminal.
    pub fn open_in_editor(&self, root: &std::path::Path) {
        let root_str = root.display().to_string();
        let editor = &self.editor;
        let terminal = &self.terminal;

        #[cfg(target_os = "macos")]
        {
            // AppleScript: open new Terminal window at path running editor
            let script = format!(
                "tell application \"Terminal\" to do script \"cd '{root_str}' && {editor} .\""
            );
            std::process::Command::new("osascript")
                .args(["-e", &script])
                .spawn()
                .ok();
        }

        #[cfg(target_os = "windows")]
        {
            // Windows Terminal (wt) uses -d flag; other terminals use cmd start /d
            if terminal.eq_ignore_ascii_case("wt") || terminal.eq_ignore_ascii_case("wt.exe") {
                std::process::Command::new("wt")
                    .args(["-d", &root_str, editor, "."])
                    .spawn()
                    .ok();
            } else {
                // cmd /c start uses /d for working directory; empty string is the window title
                std::process::Command::new("cmd")
                    .args(["/c", "start", "", terminal.as_str(), "/d", &root_str, editor, "."])
                    .spawn()
                    .ok();
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            // Linux — try terminal-specific working-directory flags
            match terminal.as_str() {
                "foot" => {
                    std::process::Command::new("foot")
                        .args(["--working-directory", &root_str, editor, "."])
                        .spawn()
                        .ok();
                }
                "kitty" => {
                    std::process::Command::new("kitty")
                        .args(["--directory", &root_str, editor, "."])
                        .spawn()
                        .ok();
                }
                "alacritty" => {
                    std::process::Command::new("alacritty")
                        .args(["--working-directory", &root_str, "-e", editor, "."])
                        .spawn()
                        .ok();
                }
                "wezterm" => {
                    std::process::Command::new("wezterm")
                        .args(["start", "--cwd", &root_str, "--", editor, "."])
                        .spawn()
                        .ok();
                }
                _ => {
                    // Generic: try -e flag
                    std::process::Command::new(terminal)
                        .args(["-e", &format!("cd '{root_str}' && {editor} .")])
                        .spawn()
                        .ok();
                }
            }
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("config.toml"))
}
