//! Minimal `clangd` LSP client for hover-type lookups.
//!
//! Spawns `clangd` as a background subprocess and talks JSON-RPC over its
//! stdin/stdout using the standard `Content-Length`-framed transport (see
//! the LSP spec's base protocol). Only the subset needed for hover is
//! implemented: `initialize`, `textDocument/didOpen`, and
//! `textDocument/hover`. Diagnostics, completion, etc. are out of scope.
//!
//! `clangd` reads project structure from `compile_commands.json` (see
//! [`newc_core::export::write_compile_commands`]) — without one it falls
//! back to a generic single-file view, which still works for hover but
//! won't see project-wide includes.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use serde_json::{Value, json};

/// A hover result string (markdown or plaintext, as reported by clangd).
#[derive(Debug, Clone)]
pub struct HoverResult {
    pub text: String,
}

enum ClientCmd {
    DidOpen {
        uri: String,
        text: String,
    },
    Hover {
        uri: String,
        line: u32,
        character: u32,
    },
}

/// Handle to a running `clangd` process. Dropping this does not terminate
/// the child — callers that want a clean shutdown should send `exit`/kill
/// explicitly (not implemented; `newc` is short-lived enough that this is
/// acceptable, matching `BuildRunner`'s lifecycle).
pub struct LspClient {
    command_sender: Sender<ClientCmd>,
    hover_receiver: Receiver<HoverResult>,
    _child: Child,
}

impl LspClient {
    /// Spawn `clangd` rooted at `project_root`. Returns `None` if `clangd`
    /// is not on PATH or fails to start.
    pub fn spawn(project_root: &Path) -> Option<Self> {
        let mut child = Command::new("clangd")
            .arg("--background-index")
            .current_dir(project_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;

        let (command_sender, command_receiver) = mpsc::channel::<ClientCmd>();
        let (hover_sender, hover_receiver) = mpsc::channel::<HoverResult>();

        thread::spawn(move || reader_loop(stdout, hover_sender));

        let root_uri = format!("file://{}", project_root.display());
        thread::spawn(move || writer_loop(stdin, command_receiver, root_uri));

        Some(Self {
            command_sender,
            hover_receiver,
            _child: child,
        })
    }

    /// Notify clangd that a file is open with the given full text.
    pub fn did_open(&self, uri: String, text: String) {
        let _ = self.command_sender.send(ClientCmd::DidOpen { uri, text });
    }

    /// Request hover info at a 0-based `(line, character)` position.
    pub fn hover(&self, uri: String, line: u32, character: u32) {
        let _ = self.command_sender.send(ClientCmd::Hover {
            uri,
            line,
            character,
        });
    }

    /// Drain the most recent hover result, if any arrived since the last call.
    pub fn try_recv_hover(&self) -> Option<HoverResult> {
        self.hover_receiver.try_recv().ok()
    }
}

fn write_message(stdin: &mut ChildStdin, value: &Value) -> std::io::Result<()> {
    let body = serde_json::to_string(value)?;
    write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    stdin.flush()
}

fn writer_loop(mut stdin: ChildStdin, command_receiver: Receiver<ClientCmd>, root_uri: String) {
    let init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {},
        }
    });
    if write_message(&mut stdin, &init).is_err() {
        return;
    }
    let initialized = json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} });
    let _ = write_message(&mut stdin, &initialized);

    let mut next_id: i64 = 2;
    for client_cmd in command_receiver {
        let request_payload = match client_cmd {
            ClientCmd::DidOpen { uri, text } => json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": { "uri": uri, "languageId": "c", "version": 1, "text": text }
                }
            }),
            ClientCmd::Hover {
                uri,
                line,
                character,
            } => {
                let id = next_id;
                next_id += 1;
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "textDocument/hover",
                    "params": {
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": character }
                    }
                })
            }
        };
        if write_message(&mut stdin, &request_payload).is_err() {
            return;
        }
    }
}

/// Read one `Content-Length`-framed JSON-RPC message from `reader`.
/// Returns `None` on EOF or a malformed frame.
fn read_message(reader: &mut impl BufRead) -> Option<Value> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).ok()?;
        if n == 0 {
            return None; // EOF
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break; // blank line ends the header block
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = value.trim().parse().ok();
        }
    }
    let len = content_length?;
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn reader_loop(stdout: ChildStdout, hover_sender: Sender<HoverResult>) {
    let mut reader = BufReader::new(stdout);
    while let Some(response_payload) = read_message(&mut reader) {
        let Some(result) = response_payload.get("result") else {
            continue;
        };
        let Some(contents) = result.get("contents") else {
            continue;
        };
        let text = extract_hover_text(contents);
        if !text.is_empty() {
            let _ = hover_sender.send(HoverResult { text });
        }
    }
}

/// `contents` in a hover response can be a plain string, a `{language,
/// value}` pair, a `{kind, value}` MarkupContent, or an array of any of
/// those — normalise all shapes to a single display string.
fn extract_hover_text(contents: &Value) -> String {
    match contents {
        Value::String(text) => text.clone(),
        Value::Object(_) => contents
            .get("value")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_default(),
        Value::Array(items) => items
            .iter()
            .map(extract_hover_text)
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn write_then_read_roundtrip() {
        let value = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" });
        let body = serde_json::to_string(&value).unwrap();
        let framed = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut reader = BufReader::new(Cursor::new(framed.into_bytes()));
        let parsed = read_message(&mut reader).unwrap();
        assert_eq!(parsed["method"], "initialize");
    }

    #[test]
    fn read_message_handles_eof() {
        let mut reader = BufReader::new(Cursor::new(Vec::<u8>::new()));
        assert!(read_message(&mut reader).is_none());
    }

    #[test]
    fn extract_hover_text_string() {
        assert_eq!(extract_hover_text(&json!("hello")), "hello");
    }

    #[test]
    fn extract_hover_text_markup_content() {
        let v = json!({ "kind": "markdown", "value": "`int foo(void)`" });
        assert_eq!(extract_hover_text(&v), "`int foo(void)`");
    }

    #[test]
    fn extract_hover_text_array() {
        let v = json!([{ "language": "c", "value": "int" }, "plain"]);
        assert_eq!(extract_hover_text(&v), "int\nplain");
    }
}
