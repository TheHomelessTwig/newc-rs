//! Persistent build history stored as `.newc_builds.json` in the project root.
//!
//! Keeps up to 100 records; older entries are dropped when the limit is exceeded.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// A single build attempt recorded in the project's history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRecord {
    /// ISO 8601-formatted timestamp of the build start.
    pub timestamp: String,
    /// Makefile target that was invoked (e.g. `"all"`, `"test"`).
    pub target: String,
    /// Process exit code, or `None` if the process could not be spawned.
    pub exit_code: Option<i32>,
    /// Elapsed wall-clock time in milliseconds.
    pub duration_ms: u64,
}

impl BuildRecord {
    /// Returns `true` if the build exited with code 0.
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Formats the duration as a human-readable string (e.g. `"1.3s"`).
    pub fn duration_str(&self) -> String {
        format!("{:.1}s", self.duration_ms as f64 / 1000.0)
    }
}

/// Append a build record to the project's history file, capping at 100 entries.
pub fn append(root: &Path, record: BuildRecord) {
    let mut records = load(root);
    records.push(record);
    // Keep last 100
    if records.len() > 100 {
        let drain = records.len() - 100;
        records.drain(..drain);
    }
    let path = root.join(".newc_builds.json");
    if let Ok(json) = serde_json::to_string_pretty(&records) {
        let _ = std::fs::write(path, json);
    }
}

/// Load all build records from `.newc_builds.json`. Returns an empty vec if the file
/// does not exist or cannot be parsed.
pub fn load(root: &Path) -> Vec<BuildRecord> {
    let path = root.join(".newc_builds.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&content).unwrap_or_default()
}
