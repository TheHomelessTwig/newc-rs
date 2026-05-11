use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;

use std::sync::mpsc::{self, Receiver, SyncSender};

#[derive(Debug, Clone)]
pub struct BuildLine {
    pub text: String,
    pub kind: LineKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineKind {
    Stdout,
    Stderr,
    Info,
    Done { exit_code: Option<i32>, duration_ms: u64 },
}

#[allow(dead_code)]
pub enum BuildCommand {
    Run { target: String, cwd: PathBuf },
    Kill,
}

pub struct BuildRunner {
    pub cmd_tx: SyncSender<BuildCommand>,
    pub line_rx: Receiver<BuildLine>,
}

impl BuildRunner {
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::sync_channel::<BuildCommand>(8);
        let (line_tx, line_rx) = mpsc::sync_channel::<BuildLine>(512);
        thread::spawn(move || runner_loop(cmd_rx, line_tx));
        Self { cmd_tx, line_rx }
    }

    pub fn run(&self, target: &str, cwd: PathBuf) {
        let _ = self.cmd_tx.try_send(BuildCommand::Run {
            target: target.to_string(),
            cwd,
        });
    }

    pub fn kill(&self) {
        let _ = self.cmd_tx.try_send(BuildCommand::Kill);
    }

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
