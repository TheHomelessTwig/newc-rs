use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::project::expand_tilde;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default = "default_editor")]
    pub editor: String,
    #[serde(default = "default_scan_dirs")]
    pub scan_dirs: Vec<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            terminal: default_terminal(),
            editor: default_editor(),
            scan_dirs: default_scan_dirs(),
            theme: default_theme(),
        }
    }
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
    std::process::Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

impl AppConfig {
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

    pub fn scan_paths(&self) -> Vec<PathBuf> {
        self.scan_dirs
            .iter()
            .map(|s| expand_tilde(&PathBuf::from(s)))
            .collect()
    }

    pub fn is_dark(&self) -> bool {
        self.theme != "light"
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
            // Windows Terminal with working directory
            std::process::Command::new("cmd")
                .args(["/c", "start", terminal, "--", "/d", &root_str, editor, "."])
                .spawn()
                .ok();
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
