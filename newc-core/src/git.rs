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
