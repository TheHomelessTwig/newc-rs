use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use newc_core::{
    analysis, doc, lint, module, project_template,
    license::License,
    project::BuildSystem,
    scaffold::{self, DefaultModule, ScaffoldOptions}, stats, sync,
};

use crate::updater;

#[derive(Parser)]
#[command(
    name = "newc",
    version,
    about = "C project scaffolding tool",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new C project
    New {
        name: String,
        /// Initialise a git repo with a .gitignore
        #[arg(long)]
        git: bool,
        /// Seed main.c from a named template (e.g. calculator)
        #[arg(long, value_name = "TEMPLATE")]
        template: Option<String>,
        /// Build system to scaffold: `make` (default) or `cmake`
        #[arg(long, value_name = "SYSTEM", default_value = "make")]
        build_system: String,
        /// License to apply (SPDX id, e.g. MIT, GPL-3.0-only, Apache-2.0, BSD-3-Clause,
        /// GPL-2.0-only, Unlicense). Writes LICENSE and stamps SPDX headers into modules.
        #[arg(long, value_name = "SPDX_ID")]
        license: Option<String>,
    },
    /// Add a new module to the current project
    Add { module: String },
    /// Remove a module (interactive picker, or by name)
    Remove {
        /// Module name to remove; omit for an interactive picker
        module: Option<String>,
        /// Skip the confirmation prompt (requires a module name)
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// List all modules in the current project
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Sync .h prototypes from .c definitions
    Sync {
        /// Only sync this module; omit to sync all
        module: Option<String>,
    },
    /// List functions unreachable from main (BFS)
    Check {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove unreachable functions (with confirmation)
    Tidy {
        /// Show what would be removed without changing any files
        #[arg(long)]
        dry_run: bool,
        /// Skip the confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Print project statistics (function count, LOC per module)
    Stats {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List functions in the project or a specific module
    Funcs {
        /// Module name to filter by (omit for all)
        module: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Lint C sources for common mistakes (L001-L017); non-zero exit on findings
    Lint {
        /// Module name to lint (omit for all sources + headers)
        module: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Insert Doxygen stub comments above undocumented functions
    Doc {
        /// Module to document
        module: String,
        /// Only this function; omit to stub every undocumented function
        function: Option<String>,
    },
    /// Open the GUI, optionally jumping to a project
    Gui {
        /// Path to a newc project to open immediately
        path: Option<PathBuf>,
    },
    /// Check for and install updates from GitHub Releases
    Update {
        /// Only check — print if an update is available without installing
        #[arg(long)]
        check: bool,
    },
    /// Run a build target in the current project (auto-detects Makefile vs CMakeLists.txt)
    ///
    /// Targets: all, run, debug, release, strict, valgrind, analyse, test, clean, help.
    /// Defaults to `all` if omitted.
    Build { target: Option<String> },
    /// Run `make test` / `cmake --build build --target test` in the current project
    Test,
    /// Internal: run the GUI in-process (spawned by `newc` itself; not for direct use)
    #[command(hide = true)]
    InternalGui {
        path: Option<PathBuf>,
    },
}

pub fn run(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::New { name, git, template, build_system, license } => {
            cmd_new(&name, git, template.as_deref(), &build_system, license.as_deref())
        }
        Command::Add { module: name } => cmd_add(&name),
        Command::Remove { module, yes } => cmd_remove(module.as_deref(), yes),
        Command::List { json } => cmd_list(json),
        Command::Sync { module: name } => cmd_sync(name.as_deref()),
        Command::Check { json } => cmd_check(json),
        Command::Tidy { dry_run, yes } => cmd_tidy(dry_run, yes),
        Command::Stats { json } => cmd_stats(json),
        Command::Funcs { module, json } => cmd_funcs(module.as_deref(), json),
        Command::Lint { module, json } => cmd_lint(module.as_deref(), json),
        Command::Doc { module, function } => cmd_doc(&module, function.as_deref()),
        Command::Gui { .. } => unreachable!("GUI handled in main"),
        Command::InternalGui { .. } => unreachable!("InternalGui handled in main"),
        Command::Update { check } => cmd_update(check),
        Command::Build { target } => cmd_build(target.as_deref().unwrap_or("all")),
        Command::Test => cmd_build("test"),
    }
}

fn cmd_update(check_only: bool) -> anyhow::Result<()> {
    if check_only {
        match updater::check()? {
            Some(v) => println!(
                "Update available: v{v}  (run `newc update` to install)"
            ),
            None => println!("Up to date (v{}).", updater::current_version()),
        }
    } else {
        updater::update()?;
    }
    Ok(())
}

fn cmd_new(
    name: &str,
    git: bool,
    template: Option<&str>,
    build_system: &str,
    license: Option<&str>,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let author = scaffold::detect_author();
    let build_system = match build_system.to_lowercase().as_str() {
        "cmake" => BuildSystem::CMake,
        "make" => BuildSystem::Make,
        other => anyhow::bail!("Unknown build system '{other}', expected 'make' or 'cmake'"),
    };
    let license = match license {
        Some(id) => Some(License::from_spdx_id(id).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown license '{id}'. Supported: {}",
                License::all().iter().map(|l| l.spdx_id()).collect::<Vec<_>>().join(", ")
            )
        })?),
        None => None,
    };
    let opts = ScaffoldOptions {
        name: name.to_string(),
        git_init: git,
        author: author.clone(),
        modules: DefaultModule::all(),
        build_system,
        license,
    };
    scaffold::create_project(&opts, &current_dir)?;
    println!("Project created: {name}");

    if let Some(tpl_name) = template {
        let templates = project_template::all_templates();
        let tpl_name_lower = tpl_name.to_lowercase();
        let tpl = templates.iter().find(|t| t.name.to_lowercase() == tpl_name_lower);
        if let Some(tpl) = tpl {
            let root = current_dir.join(name);
            let builder_state = (tpl.builder)();
            let date = chrono::Local::now().format("%d/%m/%Y").to_string();
            let code = builder_state.preview(&author, &date);
            let main_c = root.join("src").join("main.c");
            std::fs::write(&main_c, code)?;
            println!("Seeded main.c from template: {}", tpl.name);
        } else {
            eprintln!("Warning: template '{tpl_name}' not found — skipping seed");
            eprintln!("Available: {}", templates.iter().map(|t| t.name).collect::<Vec<_>>().join(", "));
        }
    }
    Ok(())
}

fn cmd_add(name: &str) -> anyhow::Result<()> {
    let root = find_project_root()?;
    module::add_module(&root, name)?;
    println!("Module '{name}' created.");
    println!("Added #include \"{name}.h\" to src/main.c");
    Ok(())
}

fn cmd_remove(module_name: Option<&str>, yes: bool) -> anyhow::Result<()> {
    let root = find_project_root()?;
    let modules = module::list_modules(&root)?;

    if modules.is_empty() {
        println!("No modules found.");
        return Ok(());
    }

    let name = match module_name {
        Some(n) => {
            if !modules.iter().any(|m| m.name == n) {
                anyhow::bail!(
                    "No module named '{n}'. Available: {}",
                    modules.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")
                );
            }
            n.to_string()
        }
        None => {
            println!("Modules:");
            for (i, m) in modules.iter().enumerate() {
                println!("  {}. {}", i + 1, m.name);
            }

            print!("Enter number to remove: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice: usize = input.trim().parse().unwrap_or(0);

            if choice < 1 || choice > modules.len() {
                println!("Invalid selection.");
                return Ok(());
            }
            modules[choice - 1].name.clone()
        }
    };

    if module_name.is_some() && !yes {
        print!("Remove module '{name}'? (Y/N): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let confirm = input.trim();
        if confirm != "Y" && confirm != "y" {
            println!("Aborted.");
            return Ok(());
        }
    }

    module::remove_module(&root, &name)?;
    println!("Removed module '{name}'.");
    Ok(())
}

fn cmd_list(json: bool) -> anyhow::Result<()> {
    let root = find_project_root()?;
    let modules = module::list_modules(&root)?;

    if json {
        let out: Vec<_> = modules
            .iter()
            .map(|m| serde_json::json!({ "name": m.name, "functions": m.function_count }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    if modules.is_empty() {
        println!("No modules found.");
        return Ok(());
    }

    println!("Modules:");
    for (i, m) in modules.iter().enumerate() {
        println!("  {}. {}", i + 1, m.name);
    }
    Ok(())
}

fn cmd_sync(module_name: Option<&str>) -> anyhow::Result<()> {
    let root = find_project_root()?;
    let messages = match module_name {
        Some(name) => {
            sync::sync_module(&root, name)?;
            vec![format!("Synced include/{name}.h")]
        }
        None => sync::sync_all(&root)?,
    };
    for msg in messages {
        println!("{msg}");
    }
    Ok(())
}

fn cmd_check(json: bool) -> anyhow::Result<()> {
    let root = find_project_root()?;
    if json {
        let unreachable = match analysis::check(&root) {
            Ok(v) => v,
            Err(newc_core::error::NewcError::NoModules) => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        let out: Vec<_> = unreachable
            .iter()
            .map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "source": f.source.display().to_string(),
                    "static": f.is_static,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }
    match analysis::check(&root) {
        Ok(unreachable) if unreachable.is_empty() => {
            println!("All module functions are reachable from main.");
        }
        Ok(unreachable) => {
            println!("Unreachable functions:");
            for f in &unreachable {
                let tag = if f.is_static { " [static]" } else { "" };
                println!("  {:<30} ({}){tag}", f.name, f.source.display());
            }
            println!("\n{} unreachable function(s) found.", unreachable.len());
        }
        Err(newc_core::error::NewcError::NoModules) => {
            println!("No module functions found.");
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

fn cmd_tidy(dry_run: bool, yes: bool) -> anyhow::Result<()> {
    let root = find_project_root()?;
    let unreachable = match analysis::check(&root) {
        Ok(v) => v,
        Err(newc_core::error::NewcError::NoModules) => {
            println!("No module functions found.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if unreachable.is_empty() {
        println!("Nothing to tidy — all functions are reachable from main.");
        return Ok(());
    }

    println!("The following unreachable functions will be removed:");
    for f in &unreachable {
        let tag = if f.is_static { " [static]" } else { "" };
        println!("  {:<30} ({}){tag}", f.name, f.source.display());
    }

    if dry_run {
        println!("\nDry run — no files changed.");
        return Ok(());
    }

    if !yes {
        println!();
        print!("Continue? (Y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let confirm = input.trim();
        if confirm != "Y" && confirm != "y" {
            println!("Aborted.");
            return Ok(());
        }
    }

    let names: Vec<String> = unreachable.iter().map(|f| f.name.clone()).collect();
    let log = analysis::tidy(&root, &names)?;
    for line in log {
        println!("{line}");
    }
    Ok(())
}

fn cmd_stats(json: bool) -> anyhow::Result<()> {
    let root = find_project_root()?;
    let project_stats = stats::compute(&root);
    if json {
        let out = serde_json::json!({
            "total_functions": project_stats.total_functions,
            "total_loc": project_stats.total_loc,
            "total_source_lines": project_stats.total_source_lines,
            "modules": project_stats.module_stats.iter().map(|m| serde_json::json!({
                "name": m.name,
                "functions": m.functions,
                "loc": m.loc,
                "source_lines": m.source_lines,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }
    println!("{:<20} {:>8} {:>8}", "Module", "Funcs", "LOC");
    println!("{}", "-".repeat(38));
    for m in &project_stats.module_stats {
        println!("{:<20} {:>8} {:>8}", m.name, m.functions, m.loc);
    }
    println!("{}", "-".repeat(38));
    println!("{:<20} {:>8} {:>8}", "TOTAL", project_stats.total_functions, project_stats.total_loc);
    println!("Source lines (incl. main): {}", project_stats.total_source_lines);
    Ok(())
}

fn cmd_funcs(module_filter: Option<&str>, json: bool) -> anyhow::Result<()> {
    let root = find_project_root()?;
    let src_dir = root.join("src");
    let Ok(entries) = std::fs::read_dir(&src_dir) else {
        println!("No src/ directory found.");
        return Ok(());
    };
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("c")
                && e.file_name() != "main.c"
        })
        .map(|e| e.path())
        .collect();
    paths.sort();
    let mut any = false;
    let mut json_out = Vec::new();
    for path in &paths {
        let mod_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        if let Some(f) = module_filter
            && !mod_name.eq_ignore_ascii_case(f) {
                continue;
            }
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let fns = sync::extract_function_implementations(&content);
        if fns.is_empty() { continue; }
        if json {
            json_out.push(serde_json::json!({
                "module": mod_name,
                "functions": fns.iter().map(|f| f.signature.clone()).collect::<Vec<_>>(),
            }));
        } else {
            println!("[{mod_name}]");
            for f in &fns {
                println!("  {}", f.signature);
            }
        }
        any = true;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&json_out)?);
    } else if !any {
        println!("No functions found.");
    }
    Ok(())
}

fn cmd_lint(module_filter: Option<&str>, json: bool) -> anyhow::Result<()> {
    let root = find_project_root()?;

    // Collect lint targets: every .c in src/ and .h in include/
    let mut targets: Vec<(PathBuf, bool)> = Vec::new();
    for (dir, ext) in [("src", "c"), ("include", "h")] {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else { continue };
        for e in entries.filter_map(|e| e.ok()) {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) == Some(ext) {
                targets.push((path, ext == "h"));
            }
        }
    }
    targets.sort();

    let mut findings = Vec::new();
    for (path, is_header) in &targets {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        if let Some(f) = module_filter
            && !stem.eq_ignore_ascii_case(f) {
                continue;
            }
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let warnings = if *is_header {
            lint::lint_header(&content)
        } else {
            lint::lint_file(&content)
        };
        let rel = path.strip_prefix(&root).unwrap_or(path).to_path_buf();
        for w in warnings {
            findings.push((rel.clone(), w));
        }
    }

    if json {
        let out: Vec<_> = findings
            .iter()
            .map(|(path, w)| {
                serde_json::json!({
                    "file": path.display().to_string(),
                    "line": w.line_no,
                    "code": w.code,
                    "severity": severity_str(&w.severity),
                    "message": w.message,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else if findings.is_empty() {
        println!("No lint findings.");
    } else {
        for (path, w) in &findings {
            println!(
                "{}:{}: [{}] {}: {}",
                path.display(),
                w.line_no,
                w.code,
                severity_str(&w.severity),
                w.message
            );
        }
        println!("\n{} finding(s).", findings.len());
    }

    if findings.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn severity_str(sev: &lint::LintSeverity) -> &'static str {
    match sev {
        lint::LintSeverity::Error => "error",
        lint::LintSeverity::Warning => "warning",
        lint::LintSeverity::Info => "info",
    }
}

fn cmd_doc(module: &str, function: Option<&str>) -> anyhow::Result<()> {
    let root = find_project_root()?;
    let src_path = root.join("src").join(format!("{module}.c"));
    if !src_path.exists() {
        anyhow::bail!("No module named '{module}' (src/{module}.c not found)");
    }

    match function {
        Some(fname) => {
            doc::insert_stub(&root, module, fname)?;
            println!("Inserted Doxygen stub above '{fname}' in src/{module}.c");
        }
        None => {
            let content = std::fs::read_to_string(&src_path)?;
            let undocumented: Vec<String> = sync::extract_function_implementations(&content)
                .iter()
                .filter(|f| f.comment.trim().is_empty())
                .map(|f| f.name.clone())
                .collect();
            if undocumented.is_empty() {
                println!("All functions in '{module}' already have comments.");
                return Ok(());
            }
            for fname in &undocumented {
                doc::insert_stub(&root, module, fname)?;
                println!("Inserted Doxygen stub above '{fname}'");
            }
            println!("{} stub(s) inserted in src/{module}.c", undocumented.len());
        }
    }
    Ok(())
}

/// Run a logical build target (`all`, `run`, `debug`, `release`, `strict`,
/// `valgrind`, `analyse`, `test`, `clean`, `help`) against whichever build
/// system the current project uses, with the same flags the GUI build
/// runner uses (see [`newc_core::build`]).
fn cmd_build(target: &str) -> anyhow::Result<()> {
    let root = find_project_root()?;
    match BuildSystem::detect(&root) {
        BuildSystem::Make => run_make(&root, target),
        BuildSystem::CMake => run_cmake(&root, target),
    }
}

fn run_make(root: &std::path::Path, target: &str) -> anyhow::Result<()> {
    use std::process::Command;
    let status = Command::new("make")
        .arg(target)
        .current_dir(root)
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("make not found. Install Make (e.g. `winget install GnuWin32.Make` on Windows)")
            } else {
                anyhow::anyhow!("Failed to run make: {e}")
            }
        })?;
    if !status.success() {
        anyhow::bail!("Build failed (exit code {:?})", status.code());
    }
    Ok(())
}

fn run_cmake(root: &std::path::Path, target: &str) -> anyhow::Result<()> {
    use newc_core::build::{cmake_build_target, cmake_configure_args, cmake_needs_reconfigure, HELP_LINES};
    use std::process::Command;

    if target == "help" {
        for msg in HELP_LINES {
            println!("{msg}");
        }
        return Ok(());
    }

    let cmake_err = |e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!("cmake not found. Install CMake (e.g. `winget install Kitware.CMake` on Windows)")
        } else {
            anyhow::anyhow!("Failed to run cmake: {e}")
        }
    };
    let check = |status: std::process::ExitStatus| -> anyhow::Result<()> {
        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("Build failed (exit code {:?})", status.code());
        }
    };

    let cache_exists = root.join("build").join("CMakeCache.txt").exists();
    if !cache_exists || cmake_needs_reconfigure(target) {
        let args = cmake_configure_args(target);
        let status = Command::new("cmake")
            .args(["-S", ".", "-B", "build"])
            .args(&args)
            .current_dir(root)
            .status()
            .map_err(cmake_err)?;
        check(status)?;
    }

    if target == "strict" {
        let status = Command::new("cmake")
            .args(["--build", "build", "--target", "clean"])
            .current_dir(root)
            .status()
            .map_err(cmake_err)?;
        check(status)?;
    }

    let mut cmd = Command::new("cmake");
    cmd.args(["--build", "build"]).current_dir(root);
    if let Some(t) = cmake_build_target(target) {
        cmd.args(["--target", t]);
    }
    let status = cmd.status().map_err(cmake_err)?;
    check(status)
}

fn find_project_root() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    if newc_core::project::Project::is_newc_project(&current_dir) {
        return Ok(current_dir);
    }
    Err(anyhow::anyhow!(
        "Not a newc project. Run this command from the project root (directory containing src/, include/, Makefile or CMakeLists.txt)."
    ))
}
