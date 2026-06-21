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

    for name in ["Makefile", "CMakeLists.txt", "CMakePresets.json", ".gitignore"] {
        let p = project_root.join(name);
        if p.exists() {
            zip.start_file(name, options).map_err(|e| NewcError::Other(e.to_string()))?;
            zip.write_all(&std::fs::read(&p)?)?;
        }
    }

    zip.finish().map_err(|e| NewcError::Other(e.to_string()))?;
    Ok(out_path)
}

/// Write a `compile_commands.json` (the clangd/clang-tidy convention) listing
/// one compile command per `.c` file in `src/`.
///
/// Uses the same include path and language standard as the generated
/// Makefile/CMakeLists (`-std=c11 -Iinclude`) so clangd sees the same view of
/// the project as the actual build.
///
/// # Errors
/// Returns an error if `src/` cannot be read or the JSON file cannot be written.
pub fn write_compile_commands(project_root: &Path) -> Result<PathBuf> {
    let src_dir = project_root.join("src");
    let mut entries = Vec::new();

    if src_dir.is_dir() {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&src_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("c"))
            .collect();
        paths.sort();

        for path in paths {
            let file_str = path.to_string_lossy().to_string();
            entries.push(serde_json::json!({
                "directory": project_root.to_string_lossy(),
                "command": format!("cc -std=c11 -I{}/include -c {}", project_root.display(), file_str),
                "file": file_str,
            }));
        }
    }

    let out_path = project_root.join("compile_commands.json");
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| NewcError::Other(e.to_string()))?;
    std::fs::write(&out_path, json)?;
    Ok(out_path)
}

/// Bundle a project for assignment submission: same contents as
/// [`export_zip`] (source, headers, build file) plus project notes and a
/// generated report, named `<student>_<project>_A<assignment_no>.zip`.
///
/// # Errors
/// Returns an error if any file cannot be read or the archive cannot be written.
pub fn pack_submission(
    project_root: &Path,
    project_name: &str,
    student_name: &str,
    assignment_no: &str,
    dest: &Path,
) -> Result<PathBuf> {
    let safe_student = student_name.replace(char::is_whitespace, "_");
    let out_path = dest.join(format!("{safe_student}_{project_name}_A{assignment_no}.zip"));
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
            zip.write_all(&std::fs::read(&path)?)?;
        }
    }

    for name in ["Makefile", "CMakeLists.txt", "CMakePresets.json", ".gitignore"] {
        let p = project_root.join(name);
        if p.exists() {
            zip.start_file(name, options).map_err(|e| NewcError::Other(e.to_string()))?;
            zip.write_all(&std::fs::read(&p)?)?;
        }
    }

    let notes = crate::notes::load(project_root);
    if !notes.trim().is_empty() {
        zip.start_file("NOTES.md", options).map_err(|e| NewcError::Other(e.to_string()))?;
        zip.write_all(notes.as_bytes())?;
    }

    let report = crate::report::generate(project_root, project_name);
    zip.start_file("REPORT.md", options).map_err(|e| NewcError::Other(e.to_string()))?;
    zip.write_all(report.as_bytes())?;

    zip.finish().map_err(|e| NewcError::Other(e.to_string()))?;
    Ok(out_path)
}
