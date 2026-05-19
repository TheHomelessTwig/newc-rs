//! Free-form per-project notes stored in `<project>/.newc_notes`.

use std::path::Path;
use crate::error::Result;

const NOTES_FILE: &str = ".newc_notes";

/// Load the project's notes as a `String`. Returns an empty string if the file does not exist.
pub fn load(project_root: &Path) -> String {
    std::fs::read_to_string(project_root.join(NOTES_FILE)).unwrap_or_default()
}

/// Write `content` to the project's notes file.
///
/// # Errors
/// Returns an IO error if the file cannot be written.
pub fn save(project_root: &Path, content: &str) -> Result<()> {
    std::fs::write(project_root.join(NOTES_FILE), content)?;
    Ok(())
}
