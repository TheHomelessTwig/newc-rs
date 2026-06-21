# Architecture

## Overview

`newc-rs` is a Cargo workspace containing two crates:

```
newc-rs/
├── newc-core/   # pure logic — no GUI, no I/O framework
└── newc/        # binary — CLI entry point + iced 0.14 GUI
```

`newc-core` has zero GUI dependencies — it can be used as a library, tested without a display, and compiled for targets that cannot run a GUI.

---

## Crate: `newc-core`

Pure Rust logic. No dependency on `iced` or any GUI toolkit.

### Module map

| Module | Responsibility |
|---|---|
| `analysis` | BFS dead-code reachability from `main()` |
| `build` | Shared Make/CMake target logic (`cmake_configure_args`, presets, help text) |
| `build_history` | Per-project build log (JSON) |
| `config` | `AppConfig`/`ProjectConfig` — serialise/deserialise user + per-project settings |
| `coverage` | `gcov` `.gcov` report parsing (line + per-function coverage %) |
| `cref` | C standard library reference data |
| `diag` | Compiler diagnostic line parser (gcc/clang format) |
| `doc` | Doxygen stub generation + comment parsing |
| `error` | `NewcError` + `Result<T>` alias |
| `export` | ZIP bundle generation, assignment packer, `compile_commands.json` |
| `function_lib` | Function library: load/save/groups/search/dependency resolution/`detect_requires` |
| `git` | `std::process::Command` wrappers for git operations (incl. stash) |
| `grep` | Project-wide search, preview/apply replace across `.c`/`.h` files |
| `header` | `.h` file read/write with `SYNC_IGNORE_START/END` block preservation |
| `lint` | Static C linter — 17 pattern-based rules (L001–L017) + mechanical quick-fixes |
| `main_builder` | `MainBlock` enum, `MainBuilderState`, C code generation |
| `meta` | Project metadata (`course`, `assignment`, `due_date`, marks) |
| `module` | Module add/remove filesystem operations + C identifier validation |
| `notes` | Plain-text project notes read/write |
| `project` | `Project` struct, discovery, `is_newc_project()`, `BuildSystem` detection (Make/CMake) |
| `project_template` | 11 built-in project templates + builder functions |
| `refactor` | Project-wide function rename, move-between-modules |
| `report` | Markdown project report generation |
| `scaffold` | Project directory creation, Makefile/CMakeLists.txt generation, `DefaultModule` enum |
| `stats` | LOC and function-count metrics |
| `sync` | Prototype extraction and `.h` regeneration |
| `templates` | C file content for all built-in modules, Unity test harness, CMake presets |
| `user_template` | User-defined template save/load |
| `valgrind` | `--xml=yes` memcheck report parsing |

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

All variants implement `to_c()` returning correctly-indented C code. Nested blocks recurse via `indent_block()`.

**`FunctionTemplate`** (`function_lib.rs`):

```rust
pub struct FunctionTemplate {
    pub name: String,
    pub module: String,
    pub description: String,
    pub signature: String,
    pub header_code: String,
    pub impl_code: String,
    pub requires: Vec<String>,
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
    pub theme: String,
    pub clang_format_style: String,
    pub workspaces: Vec<Workspace>,
    pub code_font_size: f32,
}
```

**`ProjectConfig`** (`config.rs`, per-project overrides in `.newc_config.toml`):

```rust
pub struct ProjectConfig {
    pub editor: Option<String>,
    pub terminal: Option<String>,
    pub clang_format_style: Option<String>,
    pub build_profiles: Vec<BuildProfile>,  // { name, cflags }
}
```

---

## Crate: `newc`

The binary. Depends on `newc-core`, `iced` 0.14, `clap`.

One binary: `newc` (`src/main.rs`) — handles both CLI and GUI.

### Entry point

`main.rs`:
1. Parse CLI args via `clap`
2. Route to `cli::run(cmd)` or `launch_gui(initial_path)`
3. `launch_gui()` spawns `current_exe()` with hidden `internal-gui [path]` subcommand as a detached child; parent exits immediately, freeing the terminal
   - On Windows: `CREATE_NO_WINDOW` flag prevents a console window appearing alongside the GUI
4. Child matches `Command::InternalGui { path }` → `run_gui_inline(path)`
5. `run_gui_inline()`: detect WSL2 → configure Vulkan ICD → `iced::daemon(...).run()`

### WSL2 GPU detection

`configure_wsl2_gpu()` reads `/proc/version`. On WSL2:
- If `/dev/dri` exists (GPU passthrough): prefer `virtio_icd.json` or `gfxstream_vk_icd.json`, fall back to LLVMpipe
- If no GPU: set `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json` (LLVMpipe software Vulkan)
- `WAYLAND_DISPLAY` is always unset to avoid WSLg socket instability

### MVU architecture (`app.rs`)

`NewcApp` implements the iced MVU pattern via `iced::daemon()`:

```
iced::daemon(boot, update, view)
  .title(title_for_window)
  .theme(theme_for_window)
  .subscription(subscription)
  .run()
```

`daemon()` is the multi-window entry point — it routes `view()` and `title()` by `window::Id`. No window is created automatically; the main window is opened in `new()` via `iced::window::open()`.

### Multi-window

Three panels (Library, CRef, Snippets) can be used as sidebar drawers or detached into floating OS windows:

```
state.library_window: Option<window::Id>
state.cref_window:    Option<window::Id>
state.snippets_window: Option<window::Id>
```

- **Drawer mode**: `show_library / show_cref / show_snippets` bool; rendered inline via `central_panel()`
- **Window mode**: `OpenLibraryWindow` message → `iced::window::open()` → stores returned `Id`; `view_for_window()` routes that `Id` to the full-page view
- **⊞ detach button**: appears in each panel's header when in drawer mode; fires the open message
- `iced::window::close_events()` subscription clears the stored `Id` when the user closes a window

### State (`state.rs`)

`AppState` is a flat struct — all UI state in one place. No reactive framework; the GUI reads from state each call to `view()` and writes mutations in `update()`.

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
    CallGraph(Project),
    DependencyGraph(Project),
    FlowChart(Project),
    Settings,
    Onboarding(usize),
}
```

### Update function

`update(&mut self, message: Message) -> Task<Message>` handles all messages and returns a `Task` for async work (window open/close, file I/O results). Key flows:

```
update()
  ├── BuildLine          — append to build_lines; on Done: parse diags, update history
  ├── Navigate(view)     — set state.view; load content for the destination
  ├── OpenProject(path)  — load Project, navigate to ProjectDetail
  ├── BuildStart(target) — spawn make via BuildRunner
  ├── OpenLibraryWindow  — iced::window::open() → store Id
  ├── WindowClosed(id)   — clear matching window Id from state
  ├── FileChanged(path)  — (stub) refresh affected module
  └── … 100+ message variants
```

### View functions

Each view is a pure function returning `Element<'_, Message>`:

```rust
// Typical signature
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message>
```

Views do not perform I/O or mutation — they produce an element tree that iced diffs and renders. Side effects happen only in `update()`.

`view_for_window(&self, window: Id)` routes by window Id:
- Main window → `view_main()` (top bar + sidebar + central + status + toast overlay)
- Library/CRef/Snippets windows → their respective full-page view functions

### Async build (`build_runner.rs`)

`BuildRunner` owns a background thread that reads `Child` stdout/stderr and sends `BuildLine` values via a channel. The `subscription()` polls via `iced::time::every()` — `PollBuildOutput` drains the channel into `state.build_lines` each tick.

### clangd hover (`lsp.rs`)

`LspClient::spawn()` starts `clangd` as a background subprocess (`None` if not on `PATH`) and talks JSON-RPC over stdin/stdout using `Content-Length`-framed messages — only `initialize`, `textDocument/didOpen`, and `textDocument/hover` are implemented. A writer thread owns stdin and a reader thread owns stdout; hover results flow back via an `mpsc` channel drained each `update()` tick, same pattern as `BuildRunner`.

### Syntax highlighter (`highlight.rs`)

Hand-rolled C tokeniser producing `Vec<Span>` (text + `iced::Color`). Single-pass, O(n). Handles:
- Type keywords (`int`, `char`, `void`, …) — cyan
- Flow keywords (`if`, `return`, `while`, …) — coral
- Preprocessor directives — coral
- String/char literals — yellow
- Numeric literals — purple
- Line comments (`//`) and block comments (`/* */`) — gray
- Identifiers followed by `(` — green (function names)
- Operators — coral

Used in `module_detail.rs` via `code_view()` which groups spans into lines and renders each line as a `row![]` of `text().color()` widgets (monospace, spacing 0).

### Styling (`theme.rs`)

Central styling module — all color constants, container styles, button styles, and typography helpers. Views import `use crate::theme as th` and use `th::btn_primary`, `th::color::ACCENT`, etc. rather than hardcoding colors.

Palette: Monokai Pro dark (`BG_DEEP` #1A171C → `TEXT` #FCFCFA, accent coral `#FF6188`).

### Toast overlay

`view_main()` uses `widget::Stack` to overlay toasts bottom-right without blocking content:

```rust
Stack::new()
    .push(main_col)           // full layout
    .push(toast_overlay)      // positioned via container align + padding
```

### Persistence

`~/.config/newc/`:

| File | Contents |
|---|---|
| `config.toml` | `AppConfig` |
| `projects.toml` | Known project paths |
| `functions/<module>.toml` | User function overrides |
| `templates/<name>.toml` | User-saved project templates |

Per-project (in project root):

| File | Contents |
|---|---|
| `.newc_meta.toml` | Course, assignment, due date, marks |
| `.newc_builds.json` | Build history (last 100 records) |
| `.newc_notes.md` | Project notes |

---

## Dead-code analysis (`analysis.rs`)

BFS from `main()`:
1. Collect all function signatures from `src/*.c` via `sync::extract_signatures()`
2. Load `src/main.c` body
3. BFS: for each reachable function body, find calls to known names via simple `name(` substring match
4. Everything not reached = unreachable

Intentionally naive — does not parse AST, misses function-pointer calls. Fast and correct for direct-call patterns typical in student projects.

---

## Prototype sync (`sync.rs`)

`sync_module()`:
1. Read `src/<module>.c`; extract function signatures via `extract_function_implementations()`
2. Read `include/<module>.h`; locate `SYNC_IGNORE_START`/`SYNC_IGNORE_END` block
3. Overwrite prototype section with freshly-extracted signatures + `;`
4. Leave ignore block untouched

`extract_function_implementations()` uses a line-state machine to handle multi-line signatures and nested braces.

---

## Static linter (`lint.rs`)

Text-based, no AST. Per-line pattern checks. See README for rule descriptions (L001–L015).

---

## Function library (`function_lib.rs`)

Built-in modules embedded at compile time via `include_str!` from `assets/functions/*.toml`. User overrides loaded from `~/.config/newc/functions/` and merged.

`detect_requires(impl_code, lib)` scans implementation code for stdlib patterns and library function names, returning inferred `requires` Vec.

---

## Platform support

| Platform | GUI | CLI | Notes |
|---|---|---|---|
| Linux (X11/Wayland) | ✓ | ✓ | Full support |
| WSL2 | ✓ | ✓ | Auto-detects; configures LLVMpipe Vulkan ICD |
| macOS | ✓ | ✓ | Native window backend |
| Windows | ✓ | ✓ | `CREATE_NO_WINDOW` prevents console alongside GUI |
