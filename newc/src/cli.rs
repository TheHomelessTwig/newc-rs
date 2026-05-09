use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use newc_core::{
    analysis, main_builder::MainBuilderState, module, project_template,
    scaffold::{self, DefaultModule, ScaffoldOptions}, stats, sync,
};

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
    },
    /// Add a new module to the current project
    Add { module: String },
    /// Interactively remove a module from the current project
    Remove,
    /// List all modules in the current project
    List,
    /// Sync .h prototypes from .c definitions
    Sync {
        /// Only sync this module; omit to sync all
        module: Option<String>,
    },
    /// List functions unreachable from main (BFS)
    Check,
    /// Remove unreachable functions (with confirmation)
    Tidy,
    /// Print project statistics (function count, LOC per module)
    Stats,
    /// List functions in the project or a specific module
    Funcs {
        /// Module name to filter by (omit for all)
        module: Option<String>,
    },
    /// Open the GUI, optionally jumping to a project
    Gui {
        /// Path to a newc project to open immediately
        path: Option<PathBuf>,
    },
}

pub fn run(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::New { name, git, template } => cmd_new(&name, git, template.as_deref()),
        Command::Add { module: name } => cmd_add(&name),
        Command::Remove => cmd_remove(),
        Command::List => cmd_list(),
        Command::Sync { module: name } => cmd_sync(name.as_deref()),
        Command::Check => cmd_check(),
        Command::Tidy => cmd_tidy(),
        Command::Stats => cmd_stats(),
        Command::Funcs { module } => cmd_funcs(module.as_deref()),
        Command::Gui { .. } => unreachable!("GUI handled in main"),
    }
}

fn cmd_new(name: &str, git: bool, template: Option<&str>) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let author = scaffold::detect_author();
    let opts = ScaffoldOptions {
        name: name.to_string(),
        git_init: git,
        author: author.clone(),
        modules: DefaultModule::all(),
    };
    scaffold::create_project(&opts, &cwd)?;
    println!("Project created: {name}");

    if let Some(tpl_name) = template {
        let templates = project_template::all_templates();
        let tpl_name_lower = tpl_name.to_lowercase();
        let tpl = templates.iter().find(|t| t.name.to_lowercase() == tpl_name_lower);
        if let Some(tpl) = tpl {
            let root = cwd.join(name);
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

fn cmd_remove() -> anyhow::Result<()> {
    let root = find_project_root()?;
    let modules = module::list_modules(&root)?;

    if modules.is_empty() {
        println!("No modules found.");
        return Ok(());
    }

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

    let name = &modules[choice - 1].name;
    module::remove_module(&root, name)?;
    println!("Removed module '{name}'.");
    Ok(())
}

fn cmd_list() -> anyhow::Result<()> {
    let root = find_project_root()?;
    let modules = module::list_modules(&root)?;

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

fn cmd_check() -> anyhow::Result<()> {
    let root = find_project_root()?;
    match analysis::check(&root) {
        Ok(unreachable) if unreachable.is_empty() => {
            println!("All module functions are reachable from main.");
        }
        Ok(unreachable) => {
            println!("Unreachable functions:");
            for f in &unreachable {
                println!("  {:<30} ({})", f.name, f.source.display());
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

fn cmd_tidy() -> anyhow::Result<()> {
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
        println!("  {:<30} ({})", f.name, f.source.display());
    }
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

    let names: Vec<String> = unreachable.iter().map(|f| f.name.clone()).collect();
    let log = analysis::tidy(&root, &names)?;
    for line in log {
        println!("{line}");
    }
    Ok(())
}

fn cmd_stats() -> anyhow::Result<()> {
    let root = find_project_root()?;
    let s = stats::compute(&root);
    println!("{:<20} {:>8} {:>8}", "Module", "Funcs", "LOC");
    println!("{}", "-".repeat(38));
    for m in &s.module_stats {
        println!("{:<20} {:>8} {:>8}", m.name, m.functions, m.loc);
    }
    println!("{}", "-".repeat(38));
    println!("{:<20} {:>8} {:>8}", "TOTAL", s.total_functions, s.total_loc);
    println!("Source lines (incl. main): {}", s.total_source_lines);
    Ok(())
}

fn cmd_funcs(module_filter: Option<&str>) -> anyhow::Result<()> {
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
    for path in &paths {
        let mod_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        if let Some(f) = module_filter {
            if !mod_name.eq_ignore_ascii_case(f) {
                continue;
            }
        }
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let fns = sync::extract_function_implementations(&content);
        if fns.is_empty() { continue; }
        println!("[{mod_name}]");
        for f in &fns {
            println!("  {}", f.signature);
        }
        any = true;
    }
    if !any {
        println!("No functions found.");
    }
    Ok(())
}

fn find_project_root() -> anyhow::Result<PathBuf> {
    let cwd = env::current_dir()?;
    if newc_core::project::Project::is_newc_project(&cwd) {
        return Ok(cwd);
    }
    Err(anyhow::anyhow!(
        "Not a newc project. Run this command from the project root (directory containing src/, include/, Makefile)."
    ))
}
