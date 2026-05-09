use std::path::Path;
use std::process::Command;

use crate::error::{NewcError, Result};

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub branch: String,
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

pub fn is_repo(root: &Path) -> bool {
    root.join(".git").exists()
}

pub fn status(root: &Path) -> Option<GitStatus> {
    if !is_repo(root) {
        return Some(GitStatus { initialized: false, ..Default::default() });
    }

    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let porcelain = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut staged = 0;
    let mut unstaged = 0;
    let mut untracked = 0;
    for line in porcelain.lines() {
        if line.len() < 2 { continue; }
        let x = line.chars().next().unwrap_or(' ');
        let y = line.chars().nth(1).unwrap_or(' ');
        if x == '?' && y == '?' { untracked += 1; }
        else {
            if x != ' ' && x != '?' { staged += 1; }
            if y != ' ' && y != '?' { unstaged += 1; }
        }
    }

    Some(GitStatus { branch, staged, unstaged, untracked, initialized: true })
}

pub fn log(root: &Path, count: usize) -> Vec<GitCommit> {
    if !is_repo(root) { return Vec::new(); }

    let out = Command::new("git")
        .args(["log", &format!("--max-count={count}"), "--pretty=format:%h|%s|%an|%ar"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    out.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                Some(GitCommit {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn init(root: &Path) -> Result<()> {
    let out = Command::new("git").arg("init").current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

pub fn stage_all(root: &Path) -> Result<()> {
    let out = Command::new("git").args(["add", "-A"]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

pub fn commit(root: &Path, message: &str) -> Result<()> {
    let out = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

// ── Diff ──────────────────────────────────────────────────────────────────────

pub fn diff(root: &Path) -> String {
    if !is_repo(root) { return String::new(); }
    Command::new("git")
        .args(["diff"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
}

pub fn diff_staged(root: &Path) -> String {
    if !is_repo(root) { return String::new(); }
    Command::new("git")
        .args(["diff", "--cached"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
}

// ── Per-file staging ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
}

pub fn changed_files(root: &Path) -> Vec<ChangedFile> {
    if !is_repo(root) { return Vec::new(); }
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    out.lines()
        .filter(|l| l.len() >= 3)
        .map(|line| {
            let x = line.chars().next().unwrap_or(' ');
            let y = line.chars().nth(1).unwrap_or(' ');
            let path = line[3..].to_string();
            let untracked = x == '?' && y == '?';
            ChangedFile {
                path,
                staged: !untracked && x != ' ' && x != '?',
                unstaged: !untracked && y != ' ' && y != '?',
                untracked,
            }
        })
        .collect()
}

pub fn stage_file(root: &Path, path: &str) -> Result<()> {
    let out = Command::new("git").args(["add", path]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

pub fn unstage_file(root: &Path, path: &str) -> Result<()> {
    let out = Command::new("git").args(["restore", "--staged", path]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

// ── Branches ─────────────────────────────────────────────────────────────────

pub fn branches(root: &Path) -> Vec<String> {
    if !is_repo(root) { return Vec::new(); }
    let out = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    out.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
}

pub fn current_branch(root: &Path) -> String {
    if !is_repo(root) { return String::new(); }
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn switch_branch(root: &Path, name: &str) -> Result<()> {
    let out = Command::new("git").args(["switch", name]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

pub fn new_branch(root: &Path, name: &str) -> Result<()> {
    let out = Command::new("git").args(["switch", "-c", name]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

// ── Push / Pull ───────────────────────────────────────────────────────────────

pub fn push(root: &Path) -> Result<String> {
    let out = Command::new("git").arg("push").current_dir(root).output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if out.status.success() { Ok(text) }
    else { Err(NewcError::Other(text)) }
}

pub fn pull(root: &Path) -> Result<String> {
    let out = Command::new("git").arg("pull").current_dir(root).output()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if out.status.success() { Ok(text) }
    else { Err(NewcError::Other(text)) }
}
