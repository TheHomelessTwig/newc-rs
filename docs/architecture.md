# Architecture

## Overview

`newc-rs` is a Cargo workspace containing two crates:

```
newc-rs/
├── newc-core/   # pure logic — no GUI, no I/O framework
└── newc/        # binary — CLI entry point + egui GUI
```

The strict separation means `newc-core` can be used as a library in other tools, tested without a display, and compiled for targets that cannot run a GUI.

---

## Crate: `newc-core`

Pure Rust logic. No dependency on `egui`, `eframe`, or any GUI toolkit.

### Module map

| Module | Responsibility |
|---|---|
| `analysis` | BFS dead-code reachability from `main()` |
| `build_history` | Per-project build log (JSON) |
| `config` | `AppConfig` — serialise/deserialise user settings |
| `cref` | C standard library reference data |
| `diag` | Compiler diagnostic line parser (gcc/clang format) |
| `error` | `NewcError` + `Result<T>` alias |
| `export` | ZIP bundle generation |
| `function_lib` | Function library: load, save, groups, search, dependency resolution |
| `git` | `std::process::Command` wrappers for git operations |
| `grep` | Project-wide substring search across `.c`/`.h` files |
| `header` | `.h` file read/write with `SYNC_IGNORE_START/END` block preservation |
| `lint` | Static C linter — 9 pattern-based rules |
| `main_builder` | `MainBlock` enum, `MainBuilderState`, C code generation |
| `meta` | Project metadata (`course`, `assignment`, `due_date`, marks) |
| `module` | Module add/remove filesystem operations |
| `notes` | Plain-text project notes read/write |
| `project` | `Project` struct, discovery, `is_newc_project()` |
| `project_template` | Built-in project templates + builder functions |
| `report` | Markdown project report generation |
| `scaffold` | Project directory creation, Makefile generation |
| `stats` | LOC and function-count metrics |
| `sync` | Prototype extraction and `.h` regeneration |
| `templates` | C reference data (static arrays) |
| `user_template` | User-defined template save/load |

### Key types

**`MainBlock`** (`main_builder.rs`) — the unit of the visual Composer:

```rust
pub enum MainBlock {
    VarDecl { type_name, name, init, is_array, array_size },
    FunctionCall { func_name, args, assign_to, comment },
    Comment(String),
    RawCode(String),
    BlankLine,
    IfBlock { condition, body: Vec<MainBlock>, else_body: Vec<MainBlock> },
    WhileLoop { condition, body: Vec<MainBlock> },
    ForLoop { init, condition, increment, body: Vec<MainBlock> },
}
```

All variants implement `to_c()` which returns the correctly-indented C code string. Nested blocks recurse via `indent_block()`.

**`FunctionTemplate`** (`function_lib.rs`) — one entry in the function library:

```rust
pub struct FunctionTemplate {
    pub name: String,
    pub module: String,
    pub description: String,
    pub signature: String,   // "int clamp(int value, int min, int max)"
    pub header_code: String, // goes into .h
    pub impl_code: String,   // goes into .c
    pub requires: Vec<String>, // names of other FunctionTemplates this depends on
    pub tags: Vec<String>,
    pub notes: String,
    pub starred: bool,
}
```

**`AppConfig`** (`config.rs`):

```rust
pub struct AppConfig {
    pub terminal: String,
    pub editor: String,
    pub scan_dirs: Vec<String>,
    pub theme: String,             // "dark" | "light"
    pub clang_format_style: String,
    pub workspaces: Vec<Workspace>,
}

pub struct Workspace {
    pub name: String,
    pub paths: Vec<PathBuf>,
}
```

---

## Crate: `newc`

The binary. Depends on `newc-core`, `eframe`, `egui`, `clap`.

### Entry point

`main.rs` does four things:
1. Parse CLI args via `clap` (`Cli::parse()`)
2. Route to either `cli::run(cmd)` or `run_gui(initial_path)`
3. In `run_gui()`: detect WSL2 and set `LIBGL_ALWAYS_SOFTWARE=1` if needed
4. Start `eframe::run_native()` with `NewcApp`

### Event loop (`app.rs`)

`NewcApp` implements `eframe::App`. Its `update()` is called every frame:

```
update()
  ├── drain_build_output()        — move lines from channel → state.build_lines
  │   └── on Done: append build_history, parse diagnostics, reset timer
  ├── set_visuals()               — apply theme (dark/light) every frame
  ├── handle_shortcuts()          — Ctrl+P, ?
  ├── show_snippets_panel()       — floating window
  ├── TopBottomPanel (top bar)    — nav tabs, status, Build Output toggle
  ├── TopBottomPanel (build panel)— streaming build output / diagnostics table
  ├── SidePanel (sidebar)         — filtered project list + recents
  ├── modals (Windows)            — error, tidy confirm, remove confirm,
  │                                 save template, meta editor, import, groups
  ├── quick_search overlay
  └── CentralPanel                — dispatch on state.view (match)
```

### State (`state.rs`)

`AppState` is a flat struct holding all UI state. No reactive framework — the GUI reads directly from state each frame and writes mutations back.

The `View` enum is the router:

```rust
pub enum View {
    Home,
    CreateProject,
    FunctionLibrary,
    CReference,
    ProjectDetail(Project),
    ProjectStats(Project),
    ProjectNotes(Project),
    ModuleDetail { project: Project, module_name: String },
    HeaderEditor { project: Project, module_name: String },
    MainBuilder(Project),
    AddModule { project: Project },
    GitPanel(Project),
    BuildHistory(Project),
    UsageTracker(Project),
    MakefileEditor(Project),
    ProjectSearch(Project),
    HealthDashboard(Project),
    Settings,
}
```

`View` variants that carry a `Project` clone. When the user navigates, `app.rs` writes a new variant into `state.view`. The next frame, `CentralPanel::default()` matches on it and calls the appropriate view function.

### Async build (`build_runner.rs`)

Build output is async. `BuildRunner` owns a `std::process::Child` and a `crossbeam-channel` sender. A background thread reads stdout/stderr line-by-line and sends `BuildLine` values. `drain_build_output()` drains the channel each frame — keeping the GUI responsive regardless of build speed.

```
BuildRunner::spawn() → thread
  loop:
    read line from child stdout/stderr
    channel.send(BuildLine { text, kind })

update() each frame:
  drain channel → state.build_lines
```

### View functions

Each view is a free function in its own file:

```rust
// Pattern for full-page views that need Context (for overlays/panels)
pub fn show(ctx: &Context, state: &mut AppState)

// Pattern for inline views rendered into the CentralPanel Ui
pub fn show(ui: &mut Ui, ...) -> SomeAction
```

Views that need to trigger app-level side effects return an action enum:

```rust
pub enum ProjectAction {
    None,
    GoHome,
    RunMake(String),
    OpenModuleDetail(String),
    ExportReport,
    // …
}
```

`app.rs` matches on the returned action and performs the side effect (file I/O, navigation, spawning builds, etc.). This keeps all filesystem and subprocess calls in one place.

### Syntax highlighter (`highlight.rs`)

A hand-rolled C tokeniser that produces an `egui::text::LayoutJob`. No external crates. The tokeniser is linear (single pass, O(n)) and handles:
- C keywords (blue)
- Preprocessor directives (yellow-green)
- String and character literals (orange)
- Integer and float literals (light green)
- Line comments (`//`) and block comment detection
- Operators (light grey)

It is used for read-only display in `module_detail.rs`. The edit buffer uses `TextEdit::code_editor()` which provides monospace font but no colouring — this is intentional (coloured TextEdit is not supported by egui without a custom widget).

### Persistence

All persistent data lives under `~/.config/newc/`:

| File | Contents |
|---|---|
| `config.toml` | `AppConfig` (settings, workspaces) |
| `projects.toml` | List of known project paths |
| `recents.toml` | Last 5 opened project paths |
| `groups.toml` | User-defined function group names |
| `functions/<module>.toml` | User function overrides |
| `templates/<name>.toml` | User-saved project templates |

Per-project data lives in the project root:

| File | Contents |
|---|---|
| `.newc_meta.toml` | Course, assignment, due date, marks |
| `.newc_builds.json` | Build history (last 100 records) |
| `.newc_notes.md` | Project notes (plain text) |

---

## Dead-code analysis (`analysis.rs`)

Uses BFS from `main()` to determine which module functions are reachable:

1. Collect all function signatures from `src/*.c` (excluding `main.c`) via `sync::extract_signatures()`
2. Load `src/main.c` body text
3. BFS: starting from `main`, for each function body find calls to known function names via `is_called_in()` (simple substring match `name(`)
4. Mark reachable; everything remaining is unreachable

This is intentionally naive — it does not parse a full AST, so it will miss calls through function pointers. It is fast and correct for the typical university assignment pattern of direct calls.

---

## Prototype sync (`sync.rs`)

`sync_module()` does:
1. Read `src/<module>.c`; extract all function signatures via regex
2. Read `include/<module>.h`; locate the `SYNC_IGNORE_START`/`SYNC_IGNORE_END` block (user-protected region for structs/typedefs)
3. Overwrite the prototype section of the header with freshly-extracted signatures + `;`
4. Leave the ignore block untouched

`extract_function_implementations()` uses a line-state machine rather than regex to correctly handle multi-line signatures and nested braces.

---

## Static linter (`lint.rs`)

Text-based, no AST. Each rule is a simple pattern check on each source line:

| Rule | Pattern |
|---|---|
| L001 | Line contains `gets(` |
| L002 | Line contains `strcpy(` but not `strncpy(` |
| L003 | Line contains `scanf(` and `"%s"` |
| L004 | `printf(` followed by a non-`"` character |
| L005 | `if`/`while` condition contains a single `=` |
| L006 | Line contains `sprintf(` but not `snprintf(` |
| L007 | `malloc`/`calloc`/`realloc` with no NULL check in next 4 lines |
| L008 | Integer literal > 9 not in the common-constants allow-list |
| L009 | `fopen(` with no `fclose(` in next 20 lines |

Rules are applied per-file and per-function (in module detail view) giving immediate feedback without needing a build.

---

## Platform support

| Platform | GUI | CLI | Notes |
|---|---|---|---|
| Linux (X11) | ✓ | ✓ | Full support |
| Linux (Wayland) | ✓ | ✓ | Full support |
| WSL2 | ✓ | ✓ | Auto-detects via `/proc/version`; forces Mesa software rendering |
| macOS | ✓ | ✓ | Uses native window backend |
| Windows | ✓ | ✓ | Uses native window backend; `where` replaces `which` |

Platform-conditional compilation:
- `eframe` features `wayland` and `x11` only compiled on Linux (`cfg(target_os = "linux")`)
- `default_terminal()` returns platform-appropriate defaults
- `open_in_editor()` uses AppleScript on macOS, `cmd /c start` on Windows, terminal-specific flags on Linux
- `which()` uses `where` on Windows, `which` on Unix/macOS
- WSL2 detection reads `/proc/version` — gracefully returns `false` on macOS/Windows where the file does not exist
