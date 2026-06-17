//! Project export utilities.
//!
//! Currently supports bundling a project as a ZIP archive containing all
//! source files, headers, the Makefile, and `.gitignore`.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{NewcError, Result};

/// Bundle a project's source files into a ZIP archive at `dest`.
///
/// Includes `src/*.c`, `include/*.h`, the build file (`Makefile` or `CMakeLists.txt`),
/// and `.gitignore` (if present).
///
/// # Returns
/// The path of the created ZIP file (`<dest>/<project_name>.zip`).
///
/// # Errors
/// Returns an error if any file cannot be read or the archive cannot be written.
pub fn export_zip(project_root: &Path, project_name: &str, dest: &Path) -> Result<PathBuf> {
    let out_path = dest.join(format!("{project_name}.zip"));
    let file = std::fs::File::create(&out_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let dirs_to_bundle = ["src", "include"];
    for dir in dirs_to_bundle {
        let dir_path = project_root.join(dir);
        if !dir_path.is_dir() { continue; }
        for entry in std::fs::read_dir(&dir_path)?.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }
            let rel = format!("{}/{}", dir, path.file_name().unwrap_or_default().to_string_lossy());
            zip.start_file(&rel, options).map_err(|e| NewcError::Other(e.to_string()))?;
            let data = std::fs::read(&path)?;
            zip.write_all(&data)?;
        }
    }

    for name in ["Makefile", "CMakeLists.txt", ".gitignore"] {
        let p = project_root.join(name);
        if p.exists() {
            zip.start_file(name, options).map_err(|e| NewcError::Other(e.to_string()))?;
            zip.write_all(&std::fs::read(&p)?)?;
        }
    }

    zip.finish().map_err(|e| NewcError::Other(e.to_string()))?;
    Ok(out_path)
}
