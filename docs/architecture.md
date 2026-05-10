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
| `function_lib` | Function library: load/save/groups/search/dependency resolution/`detect_requires` |
| `git` | `std::process::Command` wrappers for git operations |
| `grep` | Project-wide substring search across `.c`/`.h` files |
| `header` | `.h` file read/write with `SYNC_IGNORE_START/END` block preservation |
| `lint` | Static C linter — 15 pattern-based rules (L001–L015) |
| `main_builder` | `MainBlock` enum, `MainBuilderState`, C code generation |
| `meta` | Project metadata (`course`, `assignment`, `due_date`, marks) |
| `module` | Module add/remove filesystem operations + C identifier validation |
| `notes` | Plain-text project notes read/write |
| `project` | `Project` struct, discovery, `is_newc_project()` |
| `project_template` | 11 built-in project templates + builder functions |
| `report` | Markdown project report generation |
| `scaffold` | Project directory creation, Makefile generation, `DefaultModule` enum |
| `stats` | LOC and function-count metrics |
| `sync` | Prototype extraction and `.h` regeneration |
| `templates` | C file content for all built-in modules (including test Makefile target) |
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
    pub requires: Vec<String>, // stdlib headers or names of other FunctionTemplates
    pub tags: Vec<String>,
    pub notes: String,
    pub starred: bool,
}
```

**`DefaultModule`** (`scaffold.rs`) — modules selectable at project creation time:

```rust
pub enum DefaultModule {
    Input,
    Math,
    Display,
    Array,
    Strings,
    LinkedList,
    Files,
    TestUtils,
}
```

When `TestUtils` is selected, the generated `Makefile` includes a `test:` target that compiles `src/test_main.c` and runs the resulting binary.

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

One binary is built: `newc` (`src/main.rs`) — handles both CLI and GUI.

### Entry point

`main.rs` does:
1. Parse CLI args via `clap` (`Cli::parse()`)
2. Route to either `cli::run(cmd)` or `launch_gui(initial_path)`
3. `launch_gui()` spawns `current_exe()` with the hidden `internal-gui [path]` subcommand as a detached child process; the parent exits immediately, freeing the terminal
   - On Windows: `CREATE_NO_WINDOW` process creation flag suppresses the console window in the child, so only the GUI window appears
4. The child process matches `Command::InternalGui { path }` → calls `run_gui_inline(path)`
5. `run_gui_inline()`: detect WSL2, set `LIBGL_ALWAYS_SOFTWARE=1` if needed, start `eframe::run_native()`

### Shared state (`app.rs`)

Multi-viewport floating windows (Library, C Reference, Snippets) require `'static` closures for `ctx.show_viewport_deferred()`. These closures cannot borrow from `NewcApp` directly, so mutable state shared with them is held in `Arc<Mutex<SharedTools>>`:

```rust
pub struct SharedTools {
    pub function_lib: FunctionLibrary,
    pub library_state: LibraryState,
    pub cref_state: CrefState,
    pub snippets_state: SnippetsState,
    pub lib_action_queue: Vec<LibraryAction>,
    pub close_library: bool,
    pub close_cref: bool,
    pub close_snippets: bool,
}
```

`NewcApp` holds `tools: Arc<Mutex<SharedTools>>`. Each viewport closure captures a clone of `Arc`. Communication back to the main thread uses the action queue and close flags, which are drained in `update()` after locking.

### Event loop (`app.rs`)

`NewcApp` implements `eframe::App`. Its `update()` is called every frame:

```
update()
  ├── drain_build_output()        — move lines from channel → state.build_lines
  │   └── on Done: append build_history, parse diagnostics, reset timer
  ├── set_visuals()               — apply theme (dark/light) every frame
  ├── handle_shortcuts()          — Ctrl+P, ?
  ├── handle_library_action()     — process queued LibraryAction items from viewports
  ├── viewport management         — open/close Library, CRef, Snippets OS windows
  ├── TopBottomPanel (top bar)    — Home, New Project, Library, C Ref, Snippets, Settings
  ├── TopBottomPanel (build panel)— streaming build output / clickable diagnostics table
  ├── SidePanel (sidebar)         — filtered project list + recents
  ├── modals (Windows)            — error, tidy confirm, remove confirm,
  │                                 save template, meta editor, import, groups
  ├── quick_search overlay
  └── CentralPanel                — dispatch on state.view (match)
                                    diagnostic click → navigate to ModuleDetail + highlight line
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
    Snippets,
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
pub fn show(ctx: &Context, state: &mut AppState, tools: &mut SharedTools)

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

### Diagnostic click-through

`build_panel::show()` returns `Option<(String, usize)>` — the source filename and line number when a diagnostic row is clicked. `app.rs` receives this, finds the matching module in the current project, and navigates to `View::ModuleDetail { … }`. `ModuleDetailState.highlight_line: Option<usize>` is set; the module detail view scrolls to and marks that line with a `▶` indicator.

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
| L001 | Line contains `gets(` (not preceded by a word char — avoids `fgets`) |
| L002 | Line contains `strcpy(` but not `strncpy(` |
| L003 | Line contains `scanf(` and `"%s"` |
| L004 | `printf(` followed by a non-`"` character (not preceded by a word char — avoids `fprintf`/`snprintf`) |
| L005 | `if`/`while` condition contains a single `=` |
| L006 | Line contains `sprintf(` but not `snprintf(` |
| L007 | `malloc`/`calloc`/`realloc` with no NULL check in next 4 lines |
| L008 | Integer literal > 9 not in the common-constants allow-list |
| L009 | `fopen(` with no `fclose(` in next 20 lines |
| L010 | Header (`.h`) file missing `#ifndef` guard in first 5 non-blank lines |
| L011 | `free(ptr)` not followed by `ptr = NULL` within 3 lines |
| L012 | `strlen(` appears in a `while`/`for` loop condition |
| L013 | `atoi(` or `atof(` usage (suggest `strtol`/`strtod`) |
| L014 | `return &` with a non-static, non-global identifier (heuristic for returning local address) |
| L015 | Pointer compared to non-zero integer literal (`== 1`, `== -1`, etc.) |

L001 and L004 use a `prev_is_alpha` guard to avoid false positives from `fgets(` and `fprintf(`/`snprintf(` respectively.

`lint_header()` is a separate function that applies only L010 to `.h` files. `lint_file()` applies L001–L009 and L011–L015 to `.c` files.

Rules are applied per-file and per-function (in module detail view) giving immediate feedback without needing a build.

---

## Function library (`function_lib.rs`)

Built-in modules are embedded at compile time via `include_str!` from `assets/functions/*.toml`. The `BUILTIN_TOML_FILES` constant lists all built-in modules:

```rust
const BUILTIN_TOML_FILES: &[(&str, &str)] = &[
    ("input",       include_str!("../../assets/functions/input.toml")),
    ("math",        include_str!("../../assets/functions/math.toml")),
    ("display",     include_str!("../../assets/functions/display.toml")),
    ("array",       include_str!("../../assets/functions/array.toml")),
    ("algorithms",  include_str!("../../assets/functions/algorithms.toml")),
    ("strings",     include_str!("../../assets/functions/strings.toml")),
    ("linked_list", include_str!("../../assets/functions/linked_list.toml")),
    ("files",       include_str!("../../assets/functions/files.toml")),
    ("test_utils",  include_str!("../../assets/functions/test_utils.toml")),
];
```

`detect_requires(impl_code, lib)` scans implementation code for ~50 known stdlib patterns and all known library function names, returning a `Vec<String>` of inferred dependencies. This is used by the "Detect" button in the library editor.

---

## Updater (`updater.rs`)

`newc update` downloads the release asset from GitHub and replaces the running binary in-place:

1. Fetch latest tag from the GitHub Releases API
2. Compare against `CARGO_PKG_VERSION` via `semver_gt()`
3. Download `newc-<platform>` asset → temp file → install to `current_exe()` path
4. Attempt direct file copy; escalates to `sudo cp` if the destination is system-owned

---

## Platform support

| Platform | GUI | CLI | Notes |
|---|---|---|---|
| Linux (X11) | ✓ | ✓ | Full support |
| Linux (Wayland) | ✓ | ✓ | Full support |
| WSL2 | ✓ | ✓ | Auto-detects via `/proc/version`; forces Mesa software rendering |
| macOS | ✓ | ✓ | Uses native window backend |
| Windows | ✓ | ✓ | GUI spawned with `CREATE_NO_WINDOW` — no console alongside GUI; `where` replaces `which` |

Platform-conditional compilation:
- `eframe` features `wayland` and `x11` only compiled on Linux (`cfg(target_os = "linux")`)
- `default_terminal()` returns platform-appropriate defaults
- `open_in_editor()` uses AppleScript on macOS, `cmd /c start` on Windows, terminal-specific flags on Linux
- `which()` uses `where` on Windows, `which` on Unix/macOS
- WSL2 detection reads `/proc/version` — gracefully returns `false` on macOS/Windows where the file does not exist
