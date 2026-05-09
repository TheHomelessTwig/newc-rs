use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use crossbeam_channel::{bounded, Receiver, Sender};
use egui::Context;

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
    Done { exit_code: Option<i32> },
}

#[allow(dead_code)]
pub enum BuildCommand {
    Run { target: String, cwd: PathBuf },
    Kill,
}

pub struct BuildRunner {
    pub cmd_tx: Sender<BuildCommand>,
    pub line_rx: Receiver<BuildLine>,
}

impl BuildRunner {
    pub fn spawn(ctx: Context) -> Self {
        let (cmd_tx, cmd_rx) = bounded::<BuildCommand>(8);
        let (line_tx, line_rx) = bounded::<BuildLine>(512);

        thread::spawn(move || {
            runner_loop(cmd_rx, line_tx, ctx);
        });

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

fn runner_loop(cmd_rx: Receiver<BuildCommand>, line_tx: Sender<BuildLine>, ctx: Context) {
    for cmd in cmd_rx {
        match cmd {
            BuildCommand::Kill => {}
            BuildCommand::Run { target, cwd } => {
                let _ = line_tx.send(BuildLine {
                    text: format!("$ make {target}"),
                    kind: LineKind::Info,
                });
                ctx.request_repaint();

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
                            text: format!("Error spawning make: {e}"),
                            kind: LineKind::Stderr,
                        });
                        let _ = line_tx.send(BuildLine {
                            text: String::new(),
                            kind: LineKind::Done { exit_code: None },
                        });
                        ctx.request_repaint();
                        continue;
                    }
                };

                let stdout = child.stdout.take().map(BufReader::new);
                let stderr = child.stderr.take().map(BufReader::new);

                let tx_out = line_tx.clone();
                let ctx_out = ctx.clone();
                let stdout_thread = stdout.map(|r| {
                    thread::spawn(move || {
                        for line in r.lines().map_while(Result::ok) {
                            let _ = tx_out.send(BuildLine { text: line, kind: LineKind::Stdout });
                            ctx_out.request_repaint();
                        }
                    })
                });

                let tx_err = line_tx.clone();
                let ctx_err = ctx.clone();
                let stderr_thread = stderr.map(|r| {
                    thread::spawn(move || {
                        for line in r.lines().map_while(Result::ok) {
                            let _ = tx_err.send(BuildLine { text: line, kind: LineKind::Stderr });
                            ctx_err.request_repaint();
                        }
                    })
                });

                if let Some(t) = stdout_thread { let _ = t.join(); }
                if let Some(t) = stderr_thread { let _ = t.join(); }

                let exit_code = child.wait().ok().and_then(|s| s.code());
                let _ = line_tx.send(BuildLine {
                    text: String::new(),
                    kind: LineKind::Done { exit_code },
                });
                ctx.request_repaint();
            }
        }
    }
}
