mod cli;
mod app;
mod state;
mod build_runner;
pub mod views;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // Auto-detect cwd as project
            let cwd = std::env::current_dir().ok();
            let initial = cwd.filter(|p| newc_core::project::Project::is_newc_project(p));
            run_gui(initial)
        }
        Some(Command::Gui { path }) => {
            let initial = path.or_else(|| {
                std::env::current_dir()
                    .ok()
                    .filter(|p| newc_core::project::Project::is_newc_project(p))
            });
            run_gui(initial)
        }
        Some(cmd) => cli::run(cmd),
    }
}

fn is_wsl2() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|s| s.to_ascii_lowercase().contains("microsoft"))
        .unwrap_or(false)
}

fn run_gui(initial_path: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    // WSL2 has no hardware EGL — force Mesa software renderer before any GL init
    if is_wsl2() {
        unsafe {
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            std::env::set_var("GALLIUM_DRIVER", "llvmpipe");
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
