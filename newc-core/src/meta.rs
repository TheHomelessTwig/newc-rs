use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMeta {
    #[serde(default)]
    pub course: String,
    #[serde(default)]
    pub assignment: String,
    #[serde(default)]
    pub due_date: String,
    #[serde(default)]
    pub max_marks: Option<u32>,
    #[serde(default)]
    pub received_marks: Option<u32>,
    #[serde(default)]
    pub notes: String,
}

impl ProjectMeta {
    pub fn is_empty(&self) -> bool {
        self.course.is_empty()
            && self.assignment.is_empty()
            && self.due_date.is_empty()
            && self.max_marks.is_none()
    }

    /// Days until due date. Negative = overdue.
    pub fn days_until_due(&self) -> Option<i64> {
        if self.due_date.is_empty() {
            return None;
        }
        let due = chrono::NaiveDate::parse_from_str(&self.due_date, "%Y-%m-%d").ok()?;
        let today = chrono::Local::now().date_naive();
        Some((due - today).num_days())
    }

    pub fn marks_display(&self) -> Option<String> {
        match (self.received_marks, self.max_marks) {
            (Some(r), Some(m)) => Some(format!("{r}/{m}")),
            (Some(r), None) => Some(format!("{r}")),
            _ => None,
        }
    }
}

pub fn load(root: &Path) -> ProjectMeta {
    let path = root.join(".newc_meta.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return ProjectMeta::default();
    };
    toml::from_str(&content).unwrap_or_default()
}

pub fn save(root: &Path, meta: &ProjectMeta) -> Result<()> {
    let path = root.join(".newc_meta.toml");
    let content = toml::to_string_pretty(meta)
        .map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}
