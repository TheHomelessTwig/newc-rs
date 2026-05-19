//! Entry point for the `newc` binary.
//!
//! Parses CLI arguments via [`cli`], then either dispatches a CLI subcommand
//! or launches the iced GUI. The GUI is always started as a detached subprocess
//! (`internal-gui`) so the terminal is released immediately; [`run_gui_inline`]
//! is invoked by that subprocess and owns the iced runtime.
//!
//! On WSL2 the GPU/Vulkan environment is configured before iced starts
//! ([`configure_wsl2_gpu`]), selecting a virtio/gfxstream ICD when a DRI device
//! is present, or falling back to software rendering otherwise.

mod cli;
mod app;
mod highlight;
mod state;
mod build_runner;
mod updater;
mod theme;
pub mod views;

use clap::Parser;
use cli::{Cli, Command};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let cwd = std::env::current_dir().ok();
            let initial = cwd.filter(|p| newc_core::project::Project::is_newc_project(p));
            launch_gui(initial)
        }
        Some(Command::Gui { path }) => {
            let initial = path.or_else(|| {
                std::env::current_dir()
                    .ok()
                    .filter(|p| newc_core::project::Project::is_newc_project(p))
            });
            launch_gui(initial)
        }
        Some(Command::InternalGui { path }) => run_gui_inline(path),
        Some(cmd) => cli::run(cmd),
    }
}

/// Spawn the GUI as a detached `internal-gui` subprocess, then return.
///
/// This releases the terminal immediately while the window continues running.
/// On Windows a `CREATE_NO_WINDOW` flag suppresses the console that would
/// otherwise flash briefly.
fn launch_gui(initial_path: Option<PathBuf>) -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("internal-gui");
    if let Some(ref path) = initial_path {
        cmd.arg(path);
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn()?;
    Ok(())
}

/// Returns `true` when running inside WSL2 (detected via `/proc/version`).
fn is_wsl2() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|s| s.to_ascii_lowercase().contains("microsoft"))
        .unwrap_or(false)
}


/// Configure Vulkan / GPU environment variables for WSL2.
///
/// Prefers hardware ICD files (`virtio_icd.json`, `gfxstream_vk_icd.json`) when
/// `/dev/dri` is present and falls back to `lvp_icd.json` (LLVMpipe) or the
/// wgpu `gl` backend + softpipe as a last resort.
fn configure_wsl2_gpu() {
    unsafe { std::env::remove_var("WAYLAND_DISPLAY"); }

    let icd_dir = std::path::Path::new("/usr/share/vulkan/icd.d");
    let has_gpu = std::path::Path::new("/dev/dri").is_dir();
    let chosen = if has_gpu {
        ["virtio_icd.json", "gfxstream_vk_icd.json"]
            .iter()
            .map(|f| icd_dir.join(f))
            .find(|p| p.exists())
            .or_else(|| {
                let lvp = icd_dir.join("lvp_icd.json");
                lvp.exists().then_some(lvp)
            })
    } else {
        let lvp = icd_dir.join("lvp_icd.json");
        lvp.exists().then_some(lvp)
    };

    unsafe {
        if let Some(icd) = chosen {
            std::env::set_var("VK_ICD_FILENAMES", icd);
            std::env::remove_var("WGPU_BACKEND");
            std::env::remove_var("LIBGL_ALWAYS_SOFTWARE");
            std::env::remove_var("GALLIUM_DRIVER");
            std::env::remove_var("MESA_LOADER_DRIVER_OVERRIDE");
        } else {
            if std::env::var("WGPU_BACKEND").is_err() {
                std::env::set_var("WGPU_BACKEND", "gl");
            }
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            std::env::set_var("MESA_LOADER_DRIVER_OVERRIDE", "softpipe");
        }
    }
}

/// Start the iced daemon and block until the window closes.
///
/// Called by the `internal-gui` subcommand. Configures the GPU environment on
/// WSL2, then hands control to `iced::daemon` with [`app::NewcApp`] as the
/// application.
fn run_gui_inline(initial_path: Option<PathBuf>) -> anyhow::Result<()> {
    use app::NewcApp;

    if is_wsl2() {
        configure_wsl2_gpu();
    }

    // iced::daemon opens no window automatically — NewcApp::new() opens the main window
    // via window::open() and returns the task.
    iced::daemon(
        move || NewcApp::new(initial_path.clone()),
        NewcApp::update,
        NewcApp::view_for_window,
    )
    .title(NewcApp::title_for_window)
    .theme(NewcApp::theme_for_window)
    .subscription(NewcApp::subscription)
    .run()
    .map_err(|e| anyhow::anyhow!("GUI error: {e}"))
}
