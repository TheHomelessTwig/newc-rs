mod cli;
mod app;
mod highlight;
mod state;
mod build_runner;
mod updater;
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
        // Spawned by launch_gui: run the GUI in-process, no console window.
        Some(Command::InternalGui { path }) => run_gui_inline(path),
        Some(cmd) => cli::run(cmd),
    }
}

/// Spawn this same binary with `--internal-gui [path]` as a detached process,
/// freeing the terminal immediately. On Windows, `CREATE_NO_WINDOW` suppresses
/// the console window in the child so only the GUI window appears.
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

fn is_wsl2() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|s| s.to_ascii_lowercase().contains("microsoft"))
        .unwrap_or(false)
}

fn run_gui_inline(initial_path: Option<PathBuf>) -> anyhow::Result<()> {
    if is_wsl2() {
        unsafe {
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            std::env::set_var("GALLIUM_DRIVER", "llvmpipe");
            std::env::remove_var("WAYLAND_DISPLAY");
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("newc")
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "newc",
        options,
        Box::new(move |cc| Ok(Box::new(app::NewcApp::new(cc, initial_path.clone())))),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {e}"))
}
