use std::path::Path;
use crate::error::Result;

const NOTES_FILE: &str = ".newc_notes";

pub fn load(project_root: &Path) -> String {
    std::fs::read_to_string(project_root.join(NOTES_FILE)).unwrap_or_default()
}

pub fn save(project_root: &Path, content: &str) -> Result<()> {
    std::fs::write(project_root.join(NOTES_FILE), content)?;
    Ok(())
}
