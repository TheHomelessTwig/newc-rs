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
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;

use std::sync::mpsc::{self, Receiver, SyncSender};

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
    Done { exit_code: Option<i32>, duration_ms: u64 },
}

#[allow(dead_code)]
pub enum BuildCommand {
    Run { target: String, cwd: PathBuf },
    Kill,
}

/// Handle to the background build thread.
///
/// Created once in [`crate::app::NewcApp::new`] via [`BuildRunner::spawn`] and
/// kept alive for the lifetime of the application.
pub struct BuildRunner {
    pub cmd_tx: SyncSender<BuildCommand>,
    pub line_rx: Receiver<BuildLine>,
}

impl BuildRunner {
    /// Spawn the background runner thread and return the channel handles.
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::sync_channel::<BuildCommand>(8);
        let (line_tx, line_rx) = mpsc::sync_channel::<BuildLine>(512);
        thread::spawn(move || runner_loop(cmd_rx, line_tx));
        Self { cmd_tx, line_rx }
    }

    /// Send a `make <target>` command to the runner thread.
    ///
    /// The previous build (if any) is not explicitly killed; the runner
    /// processes commands sequentially, so the new run starts after the
    /// previous one finishes.
    pub fn run(&self, target: &str, cwd: PathBuf) {
        let _ = self.cmd_tx.try_send(BuildCommand::Run {
            target: target.to_string(),
            cwd,
        });
    }

    /// Request the runner to abort the current build (best-effort).
    pub fn kill(&self) {
        let _ = self.cmd_tx.try_send(BuildCommand::Kill);
    }

    /// Collect all [`BuildLine`]s queued since the last call (non-blocking).
    pub fn drain(&self) -> Vec<BuildLine> {
        let mut lines = Vec::new();
        while let Ok(line) = self.line_rx.try_recv() {
            lines.push(line);
        }
        lines
    }
}

fn runner_loop(cmd_rx: mpsc::Receiver<BuildCommand>, line_tx: SyncSender<BuildLine>) {
    for cmd in cmd_rx {
        match cmd {
            BuildCommand::Kill => {}
            BuildCommand::Run { target, cwd } => {
                let _ = line_tx.send(BuildLine {
                    text: format!("$ make {target}"),
                    kind: LineKind::Info,
                });

                let start = Instant::now();

                let mut child = match Command::new("make")
                    .arg(&target)
                    .current_dir(&cwd)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = line_tx.send(BuildLine {
                            text: format!("Error: {e}"),
                            kind: LineKind::Stderr,
                        });
                        #[cfg(target_os = "windows")]
                        if e.kind() == std::io::ErrorKind::NotFound {
                            for msg in [
                                "make not found. Install one of:",
                                "  winget install GnuWin32.Make",
                                "  or MinGW-w64 (includes mingw32-make)",
                                "  https://www.mingw-w64.org/",
                            ] {
                                let _ = line_tx.send(BuildLine { text: msg.into(), kind: LineKind::Info });
                            }
                        }
                        let _ = line_tx.send(BuildLine {
                            text: String::new(),
                            kind: LineKind::Done {
                                exit_code: None,
                                duration_ms: start.elapsed().as_millis() as u64,
                            },
                        });
                        continue;
                    }
                };

                let stdout = child.stdout.take().map(BufReader::new);
                let stderr = child.stderr.take().map(BufReader::new);

                let tx_out = line_tx.clone();
                let stdout_thread = stdout.map(|r| {
                    thread::spawn(move || {
                        for line in r.lines().map_while(Result::ok) {
                            let _ = tx_out.send(BuildLine { text: line, kind: LineKind::Stdout });
                        }
                    })
                });

                let tx_err = line_tx.clone();
                let stderr_thread = stderr.map(|r| {
                    thread::spawn(move || {
                        for line in r.lines().map_while(Result::ok) {
                            let _ = tx_err.send(BuildLine { text: line, kind: LineKind::Stderr });
                        }
                    })
                });

                if let Some(t) = stdout_thread { let _ = t.join(); }
                if let Some(t) = stderr_thread { let _ = t.join(); }

                let exit_code = child.wait().ok().and_then(|s| s.code());
                let duration_ms = start.elapsed().as_millis() as u64;
                let _ = line_tx.send(BuildLine {
                    text: String::new(),
                    kind: LineKind::Done { exit_code, duration_ms },
                });
            }
        }
    }
}
