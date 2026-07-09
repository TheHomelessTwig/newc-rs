//! Per-project academic metadata stored in `<project>/.newc_meta.toml`.
//!
//! Tracks course name, assignment, due date, and marks. All fields are optional
//! so the file can remain minimal for non-academic projects.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Academic metadata associated with a newc project.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMeta {
    /// Course or unit code (e.g. `"ICT159"`).
    #[serde(default)]
    pub course: String,
    /// Assignment identifier (e.g. `"Project 2"`).
    #[serde(default)]
    pub assignment: String,
    /// Due date in `YYYY-MM-DD` format.
    #[serde(default)]
    pub due_date: String,
    /// Total marks available for this assignment.
    #[serde(default)]
    pub max_marks: Option<u32>,
    /// Marks actually received after submission.
    #[serde(default)]
    pub received_marks: Option<u32>,
    /// Free-form notes about the assignment.
    #[serde(default)]
    pub notes: String,
}

impl ProjectMeta {
    /// Returns `true` if no meaningful metadata has been set.
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

    /// Format marks as `"received/max"`, `"received"`, or `None` if no marks are recorded.
    pub fn marks_display(&self) -> Option<String> {
        match (self.received_marks, self.max_marks) {
            (Some(r), Some(m)) => Some(format!("{r}/{m}")),
            (Some(r), None) => Some(format!("{r}")),
            _ => None,
        }
    }
}

/// Load project metadata from `<root>/.newc_meta.toml`, returning defaults on any error.
pub fn load(root: &Path) -> ProjectMeta {
    let path = root.join(".newc_meta.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return ProjectMeta::default();
    };
    toml::from_str(&content).unwrap_or_default()
}

/// Persist project metadata to `<root>/.newc_meta.toml`.
///
/// # Errors
/// Returns an error if TOML serialisation or the file write fails.
pub fn save(root: &Path, meta: &ProjectMeta) -> Result<()> {
    let path = root.join(".newc_meta.toml");
    let content =
        toml::to_string_pretty(meta).map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}
