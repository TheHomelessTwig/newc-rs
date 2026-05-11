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

fn run_gui_inline(initial_path: Option<PathBuf>) -> anyhow::Result<()> {
    use app::NewcApp;
    use iced::window;

    iced::application(
        move || NewcApp::new(initial_path.clone()),
        NewcApp::update,
        NewcApp::view,
    )
    .title(NewcApp::title)
    .theme(NewcApp::theme)
    .subscription(NewcApp::subscription)
    .window(window::Settings {
        size: iced::Size::new(1100.0, 700.0),
        min_size: Some(iced::Size::new(800.0, 500.0)),
        position: window::Position::Default,
        ..Default::default()
    })
    .run()
    .map_err(|e| anyhow::anyhow!("GUI error: {e}"))
}
