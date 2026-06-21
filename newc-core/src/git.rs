//! Git operations for newc projects.
//!
//! Wraps `git` CLI commands — no libgit2 dependency. All functions that
//! spawn git processes treat a missing repository as a graceful non-error
//! by returning empty data or `false`.

use std::path::Path;
use std::process::Command;

use crate::error::{NewcError, Result};

/// Summary of a repository's working-tree state.
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    /// Current branch name, or `"unknown"` if it cannot be determined.
    pub branch: String,
    /// Number of files with staged (index) changes.
    pub staged: usize,
    /// Number of files with unstaged working-tree changes.
    pub unstaged: usize,
    /// Number of untracked files.
    pub untracked: usize,
    /// `false` if the directory is not a git repository.
    pub initialized: bool,
}

/// A single git commit from `git log`.
#[derive(Debug, Clone)]
pub struct GitCommit {
    /// Abbreviated commit hash.
    pub hash: String,
    /// Commit subject line.
    pub message: String,
    pub author: String,
    /// Relative date string as reported by git (e.g. `"3 days ago"`).
    pub date: String,
}

/// Returns `true` if `root` contains a `.git` directory.
pub fn is_repo(root: &Path) -> bool {
    root.join(".git").exists()
}

/// Return the current git status for `root`, or `None` if git is unavailable.
///
/// If the directory is not a repository the returned value has `initialized: false`
/// and zero counts for all other fields.
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

/// Return the last `count` commits from the repository's log.
///
/// Returns an empty vec if the directory is not a git repo.
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

/// Initialise a new git repository at `root`.
///
/// # Errors
/// Returns an error containing git's stderr output if `git init` fails.
pub fn init(root: &Path) -> Result<()> {
    let out = Command::new("git").arg("init").current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

/// Stage all changes in the working tree (`git add -A`).
///
/// # Errors
/// Returns an error containing git's stderr output on failure.
pub fn stage_all(root: &Path) -> Result<()> {
    let out = Command::new("git").args(["add", "-A"]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

/// Create a commit with the given message.
///
/// # Errors
/// Returns an error containing git's stderr output on failure.
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

/// Return the unstaged diff as a UTF-8 string, or an empty string on error.
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

/// Return the staged diff (`--cached`) as a UTF-8 string, or an empty string on error.
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

/// A file with at least one kind of change in the working tree.
#[derive(Debug, Clone)]
pub struct ChangedFile {
    /// Path relative to the repository root.
    pub path: String,
    /// Has index (staged) changes.
    pub staged: bool,
    /// Has working-tree (unstaged) changes.
    pub unstaged: bool,
    /// File is not tracked by git.
    pub untracked: bool,
}

/// List all files with any kind of change (staged, unstaged, or untracked).
///
/// Returns an empty vec if the directory is not a git repository.
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

/// Stage a specific file by its repository-relative path.
///
/// # Errors
/// Returns an error containing git's stderr output on failure.
pub fn stage_file(root: &Path, path: &str) -> Result<()> {
    let out = Command::new("git").args(["add", path]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

/// Unstage a specific file (`git restore --staged`).
///
/// # Errors
/// Returns an error containing git's stderr output on failure.
pub fn unstage_file(root: &Path, path: &str) -> Result<()> {
    let out = Command::new("git").args(["restore", "--staged", path]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

// ── Branches ─────────────────────────────────────────────────────────────────

/// Return a list of all local branch names.
///
/// Returns an empty vec if the directory is not a git repository.
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

/// Return the name of the currently checked-out branch, or an empty string on error.
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

/// Switch to an existing branch (`git switch <name>`).
///
/// # Errors
/// Returns an error if the branch does not exist or the switch fails.
pub fn switch_branch(root: &Path, name: &str) -> Result<()> {
    let out = Command::new("git").args(["switch", name]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

/// Create and switch to a new branch (`git switch -c <name>`).
///
/// # Errors
/// Returns an error if a branch with that name already exists or git fails.
pub fn new_branch(root: &Path, name: &str) -> Result<()> {
    let out = Command::new("git").args(["switch", "-c", name]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

// ── Push / Pull ───────────────────────────────────────────────────────────────

/// Push the current branch to its upstream remote.
///
/// # Returns
/// Combined stdout and stderr from git.
///
/// # Errors
/// Returns the combined output as an error string if the push fails.
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

// ── Stash ────────────────────────────────────────────────────────────────────

/// A single entry from `git stash list`.
#[derive(Debug, Clone)]
pub struct StashEntry {
    /// Stash reference, e.g. `"stash@{0}"`.
    pub reference: String,
    /// Description git generated for the stash (branch + commit it was made from).
    pub description: String,
}

/// Stash the current working-tree changes (`git stash push`).
///
/// # Errors
/// Returns an error containing git's stderr output on failure.
pub fn stash(root: &Path) -> Result<()> {
    let out = Command::new("git").args(["stash", "push"]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

/// Apply and drop the most recent stash (`git stash pop`).
///
/// # Errors
/// Returns an error containing git's stderr output on failure (e.g. merge conflict).
pub fn stash_pop(root: &Path) -> Result<()> {
    let out = Command::new("git").args(["stash", "pop"]).current_dir(root).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(NewcError::Other(String::from_utf8_lossy(&out.stderr).trim().to_string()))
    }
}

/// List all stash entries, most recent first.
///
/// Returns an empty vec if the directory is not a git repository.
pub fn stash_list(root: &Path) -> Vec<StashEntry> {
    if !is_repo(root) { return Vec::new(); }
    let out = Command::new("git")
        .args(["stash", "list", "--format=%gd|%gs"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    out.lines()
        .filter_map(|line| {
            let (reference, description) = line.split_once('|')?;
            Some(StashEntry { reference: reference.to_string(), description: description.to_string() })
        })
        .collect()
}

/// Pull from the upstream remote.
///
/// # Returns
/// Combined stdout and stderr from git.
///
/// # Errors
/// Returns the combined output as an error string if the pull fails.
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
