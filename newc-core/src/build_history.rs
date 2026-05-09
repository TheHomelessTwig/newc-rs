use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRecord {
    pub timestamp: String,
    pub target: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

impl BuildRecord {
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }

    pub fn duration_str(&self) -> String {
        format!("{:.1}s", self.duration_ms as f64 / 1000.0)
    }
}

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

pub fn load(root: &Path) -> Vec<BuildRecord> {
    let path = root.join(".newc_builds.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&content).unwrap_or_default()
}
