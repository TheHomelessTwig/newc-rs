//! Background build runner for executing `make` targets.
//!
//! [`BuildRunner`] owns a pair of MPSC channels: the UI sends [`BuildCommand`]s
//! (start a build, kill the current one) and receives [`BuildLine`]s (stdout,
//! stderr, and a sentinel [`LineKind::Done`] when the process exits).
//!
//! A dedicated OS thread (`runner_loop`) blocks on the command channel,
//! spawns `make` as a subprocess, and streams its output line-by-line via two
//! reader threads — one for stdout, one for stderr — before waiting for the
//! process to exit and sending a final `Done` line with the exit code and
//! elapsed milliseconds.
//!
//! The UI drains accumulated lines each frame by calling [`BuildRunner::drain`].

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Instant;

use std::sync::mpsc::{self, Receiver, SyncSender};

use newc_core::build::{
    HELP_LINES, cmake_build_target, cmake_configure_args, cmake_needs_reconfigure,
};
use newc_core::project::BuildSystem;

/// A single line of build output with its classification.
#[derive(Debug, Clone)]
pub struct BuildLine {
    pub text: String,
    pub kind: LineKind,
}

/// Classification of a [`BuildLine`].
#[derive(Debug, Clone, PartialEq)]
pub enum LineKind {
    /// Normal stdout from `make` or the compiler.
    Stdout,
    /// Stderr from `make` or the compiler (usually warnings and errors).
    Stderr,
    /// Synthetic informational line injected by the runner (e.g. `"$ make all"`).
    Info,
    /// Sentinel emitted after the process exits. `exit_code` is `None` if the
    /// process could not be started or was forcibly killed before exiting.
    Done {
        exit_code: Option<i32>,
        duration_ms: u64,
    },
}

#[allow(dead_code)]
pub enum BuildCommand {
    Run {
        target: String,
        cwd: PathBuf,
        build_system: BuildSystem,
        args: String,
        extra_cflags: String,
    },
    Kill,
}

/// Handle to the background build thread.
///
/// Created once in [`crate::app::NewcApp::new`] via [`BuildRunner::spawn`] and
/// kept alive for the lifetime of the application.
pub struct BuildRunner {
    pub command_sender: SyncSender<BuildCommand>,
    pub line_receiver: Receiver<BuildLine>,
}

impl BuildRunner {
    /// Spawn the background runner thread and return the channel handles.
    pub fn spawn() -> Self {
        let (command_sender, command_receiver) = mpsc::sync_channel::<BuildCommand>(8);
        let (line_sender, line_receiver) = mpsc::sync_channel::<BuildLine>(512);
        thread::spawn(move || runner_loop(command_receiver, line_sender));
        Self {
            command_sender,
            line_receiver,
        }
    }

    /// Send a build command (`make <target>` or the CMake equivalent) to the runner thread.
    ///
    /// The previous build (if any) is not explicitly killed; the runner
    /// processes commands sequentially, so the new run starts after the
    /// previous one finishes.
    pub fn run(
        &self,
        target: &str,
        cwd: PathBuf,
        build_system: BuildSystem,
        args: &str,
        extra_cflags: &str,
    ) {
        let _ = self.command_sender.try_send(BuildCommand::Run {
            target: target.to_string(),
            cwd,
            build_system,
            args: args.to_string(),
            extra_cflags: extra_cflags.to_string(),
        });
    }

    /// Request the runner to abort the current build (best-effort).
    pub fn kill(&self) {
        let _ = self.command_sender.try_send(BuildCommand::Kill);
    }

    /// Collect all [`BuildLine`]s queued since the last call (non-blocking).
    pub fn drain(&self) -> Vec<BuildLine> {
        let mut lines = Vec::new();
        while let Ok(line) = self.line_receiver.try_recv() {
            lines.push(line);
        }
        lines
    }
}

fn runner_loop(command_receiver: mpsc::Receiver<BuildCommand>, line_sender: SyncSender<BuildLine>) {
    for command in command_receiver {
        match command {
            BuildCommand::Kill => {}
            BuildCommand::Run {
                target,
                cwd,
                build_system,
                args,
                extra_cflags,
            } => {
                let start = Instant::now();
                let exit_code = match build_system {
                    BuildSystem::Make => {
                        run_make(&target, &cwd, &args, &extra_cflags, &line_sender)
                    }
                    BuildSystem::CMake => run_cmake(&target, &cwd, &args, &line_sender),
                };
                let duration_ms = start.elapsed().as_millis() as u64;
                let _ = line_sender.send(BuildLine {
                    text: String::new(),
                    kind: LineKind::Done {
                        exit_code,
                        duration_ms,
                    },
                });
            }
        }
    }
}

/// Run one command to completion, streaming its stdout/stderr line-by-line.
/// Returns `None` if the process failed to spawn.
fn stream_command(mut cmd: Command, line_sender: &SyncSender<BuildLine>) -> Option<i32> {
    let mut child: Child = match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = line_sender.send(BuildLine {
                text: format!("Error: {e}"),
                kind: LineKind::Stderr,
            });
            return None;
        }
    };

    let stdout = child.stdout.take().map(BufReader::new);
    let stderr = child.stderr.take().map(BufReader::new);

    let stdout_line_sender = line_sender.clone();
    let stdout_thread = stdout.map(|r| {
        thread::spawn(move || {
            for line in r.lines().map_while(Result::ok) {
                let _ = stdout_line_sender.send(BuildLine {
                    text: line,
                    kind: LineKind::Stdout,
                });
            }
        })
    });

    let stderr_line_sender = line_sender.clone();
    let stderr_thread = stderr.map(|r| {
        thread::spawn(move || {
            for line in r.lines().map_while(Result::ok) {
                let _ = stderr_line_sender.send(BuildLine {
                    text: line,
                    kind: LineKind::Stderr,
                });
            }
        })
    });

    if let Some(t) = stdout_thread {
        let _ = t.join();
    }
    if let Some(t) = stderr_thread {
        let _ = t.join();
    }

    child.wait().ok().and_then(|s| s.code())
}

fn run_make(
    target: &str,
    cwd: &std::path::Path,
    args: &str,
    extra_cflags: &str,
    line_sender: &SyncSender<BuildLine>,
) -> Option<i32> {
    let mut cmd = Command::new("make");
    cmd.arg(target).current_dir(cwd);
    let mut log = format!("$ make {target}");
    if target == "run" && !args.is_empty() {
        cmd.arg(format!("ARGS={args}"));
        log.push_str(&format!(" ARGS=\"{args}\""));
    }
    if !extra_cflags.is_empty() {
        cmd.arg(format!("EXTRA_CFLAGS={extra_cflags}"));
        log.push_str(&format!(" EXTRA_CFLAGS=\"{extra_cflags}\""));
    }
    let _ = line_sender.send(BuildLine {
        text: log,
        kind: LineKind::Info,
    });
    let code = stream_command(cmd, line_sender);

    #[cfg(target_os = "windows")]
    if code.is_none() {
        for line_text in [
            "make not found. Install one of:",
            "  winget install GnuWin32.Make",
            "  or MinGW-w64 (includes mingw32-make)",
            "  https://www.mingw-w64.org/",
        ] {
            let _ = line_sender.send(BuildLine {
                text: line_text.into(),
                kind: LineKind::Info,
            });
        }
    }
    code
}

fn run_cmake(
    target: &str,
    cwd: &std::path::Path,
    run_args: &str,
    line_sender: &SyncSender<BuildLine>,
) -> Option<i32> {
    if target == "help" {
        for line_text in HELP_LINES {
            let _ = line_sender.send(BuildLine {
                text: (*line_text).into(),
                kind: LineKind::Info,
            });
        }
        return Some(0);
    }

    let cache_exists = cwd.join("build").join("CMakeCache.txt").exists();
    // Reconfigure is also needed when ARGS for the `run` target changed, since
    // CMake substitutes ${ARGS} into the custom target's COMMAND at configure time.
    let needs_configure = !cache_exists
        || cmake_needs_reconfigure(target)
        || (target == "run" && !run_args.is_empty());
    if needs_configure {
        let mut cmd = Command::new("cmake");
        let mut log;
        if cwd.join("CMakePresets.json").exists() {
            let preset = newc_core::build::cmake_preset_name(target);
            cmd.args(["--preset", preset]).current_dir(cwd);
            log = format!("$ cmake --preset {preset}");
            if target == "run" && !run_args.is_empty() {
                cmd.arg(format!("-DARGS={run_args}"));
                log.push_str(&format!(" -DARGS=\"{run_args}\""));
            }
        } else {
            let mut args: Vec<String> = cmake_configure_args(target)
                .into_iter()
                .map(String::from)
                .collect();
            if target == "run" && !run_args.is_empty() {
                args.push(format!("-DARGS={run_args}"));
            }
            log = format!("$ cmake -S . -B build {}", args.join(" "));
            cmd.args(["-S", ".", "-B", "build"])
                .args(&args)
                .current_dir(cwd);
        }
        let _ = line_sender.send(BuildLine {
            text: log,
            kind: LineKind::Info,
        });
        match stream_command(cmd, line_sender) {
            Some(0) => {}
            other => return other,
        }
    }

    if target == "strict" {
        let _ = line_sender.send(BuildLine {
            text: "$ cmake --build build --target clean".into(),
            kind: LineKind::Info,
        });
        let mut clean_cmd = Command::new("cmake");
        clean_cmd
            .args(["--build", "build", "--target", "clean"])
            .current_dir(cwd);
        if stream_command(clean_cmd, line_sender) != Some(0) {
            return Some(1);
        }
    }

    let build_target = cmake_build_target(target);
    let mut cmd = Command::new("cmake");
    cmd.args(["--build", "build"]).current_dir(cwd);
    if let Some(t) = build_target {
        cmd.args(["--target", t]);
    }
    let _ = line_sender.send(BuildLine {
        text: format!(
            "$ cmake --build build{}",
            build_target
                .map(|t| format!(" --target {t}"))
                .unwrap_or_default()
        ),
        kind: LineKind::Info,
    });
    stream_command(cmd, line_sender)
}
