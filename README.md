# newc

A Rust GUI application for scaffolding, building, and managing C projects. Replaces a hand-rolled bash script with a fully-featured desktop tool built on [egui](https://github.com/emilk/egui).

![Version](https://img.shields.io/badge/version-0.3.0-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)

---

## Features

### Project Management
- **Scaffold** new C projects (`src/`, `include/`, `build/`, `Makefile`) with one command or via the GUI
- **Module system** — add/remove `.c`/`.h` pairs; auto-sync prototypes between source and header
- **Workspaces** — organise projects into named groups (e.g. by course or client); archive old projects
- **Project metadata** — attach course name, assignment title, due date, and marks; due-date countdown shown in the project header
- **Export** — bundle a project as a ZIP, or generate a Markdown report (stats, module list, build history, notes)
- **Recent projects** — last 5 opened shown in the sidebar for quick access

### Build & Analysis
- **One-click make** — run `all`, `run`, `debug`, `release`, `strict`, or `clean` targets from the GUI
- **Live build output** — streaming stdout/stderr, with timing ("Build succeeded in 1.4s")
- **Build history** — per-project log of every build (target, result, duration) stored in `.newc_builds.json`
- **Compiler diagnostics** — gcc/clang output parsed into a colour-coded table after each build; click a row to navigate to the source file
- **Dead-code analysis** — BFS reachability from `main()`; unreachable functions highlighted in orange
- **Health dashboard** — single view of dead-code count, missing includes, TODO/FIXMEs, lint warnings, and last build status

### Code Intelligence
- **Static linter** — 9 rules catching common C mistakes:
  - `L001` — `gets()` usage (unsafe)
  - `L002` — `strcpy()` without bounds
  - `L003` — `scanf("%s")` without width limit
  - `L004` — `printf()` with non-literal format string
  - `L005` — assignment in condition (`if (x = y)`)
  - `L006` — `sprintf()` without bounds
  - `L007` — `malloc`/`calloc` result not checked for `NULL`
  - `L008` — magic number literals
  - `L009` — `fopen()` with no matching `fclose()`
- **Syntax highlighting** — C keywords, strings, numbers, comments, and operators colour-coded in the module editor
- **Missing include detection** — warns when a library function is called but its `.h` is not included; one-click auto-fix
- **Prototype mismatch detection** — banner shown after saving a function if the `.c` signature diverges from the `.h`
- **Function call tree** — recursive call graph (up to depth 5) rendered inline for any selected function
- **Project-wide search** — grep across all `.c` and `.h` files with highlighted match display and click-to-navigate

### Function Library
- **Built-in library** — `input`, `math`, `display`, `array`, and `algorithms` modules (~70+ functions)
- **Import from `.c`** — extract functions from any existing source file and add to the library
- **Favourites** — star/unstar functions; "★ Starred" filter group in the sidebar
- **Custom groups** — create, rename, and delete function groups
- **User overrides** — stored in `~/.config/newc/functions/` as TOML files
- **Usage tracker** — per-project view showing which library functions are used in which source files

### main() Composer
A visual block-based builder for `src/main.c`:
- Block types: **Variable declaration**, **Function call** (with param-by-param argument fields), **Comment**, **Raw C**, **Blank line**, **If block**, **While loop**, **For loop**
- Nested control flow with inline child-block editing (add/remove/reorder children without leaving the parent)
- Global variable declarations
- `#include` checkbox list (auto-populated from project headers)
- Undo/redo (50-level history, Ctrl+Z/Y)
- Duplicate any block (⧉)
- Live preview of generated `src/main.c`
- Write to file with one click

### Git Integration
- Status view: branch, staged/unstaged/untracked file counts
- **Per-file staging** — checkbox per changed file; stage or unstage individual files
- **Diff view** — unified diff with colour-coded +/- lines; toggle between staged and unstaged
- **Branch management** — ComboBox to switch branches; create new branches
- **Commit** — message input + Commit button
- **Push / Pull** — output shown in build panel
- Init a new repository from the GUI

### Code Snippets
17 built-in C patterns available in a floating panel (toggle from top bar):
for loop, while, do-while, switch, if/else chain, struct definition, malloc/free, fopen/fclose, fgets loop, printf format reference, string functions, array init, qsort, function pointer, argc/argv main, enum definition, memset

### Templates
7 built-in project templates with pre-seeded `main()`:
- Calculator, Array Processor, Grade Manager, Menu-Driven App, File Parser, Linked List, Student Records

Create custom templates by saving any project's structure.

### C Reference
Searchable reference for ~70 C standard library functions across `stdio.h`, `stdlib.h`, `string.h`, `math.h`, and `time.h` — with signatures, descriptions, and usage examples.

### clang-format Integration
Format any function directly in the editor using clang-format. Configurable style (file, LLVM, Google, Chromium, GNU, Microsoft) via Settings.

### Self-update
```bash
newc update           # check for and install the latest release
newc update --check   # print whether an update is available without installing
```
Downloads the correct pre-built binary for your platform from GitHub Releases. Escalates to `sudo` automatically if the install path is system-owned.

---

## Documentation

| Document | Description |
|---|---|
| [Architecture](docs/architecture.md) | System design, module map, key types, event loop, data flow |
| [Data Formats](docs/data-formats.md) | All TOML/JSON schemas with annotated examples |
| [Building](docs/building.md) | Platform-specific build instructions (Linux, macOS, Windows, WSL2) |
| [Contributing](docs/contributing.md) | Dev setup, adding features, lint rules, templates, commit style |

---

## Installation

### Pre-built binaries (no Rust required)

Download the binary for your platform from [GitHub Releases](https://github.com/TheHomelessTwig/newc-rs/releases/latest):

| Platform | File |
|---|---|
| Linux x86_64 | `newc-x86_64-linux` |
| Linux aarch64 | `newc-aarch64-linux` |
| macOS Intel | `newc-x86_64-macos` |
| macOS Apple Silicon | `newc-aarch64-macos` |
| Windows x86_64 | `newc-x86_64-windows.exe` |

```bash
# Linux example
curl -fsSL https://github.com/TheHomelessTwig/newc-rs/releases/latest/download/newc-x86_64-linux \
    -o /tmp/newc && chmod +x /tmp/newc && sudo mv /tmp/newc /usr/local/bin/newc
```

Once installed, keep up to date with:
```bash
newc update
```

### Build from source

Requires Rust stable (via [rustup](https://rustup.rs/)), `gcc`/`clang`, and `make`.

```bash
git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
sudo cp target/release/newc /usr/local/bin/newc
```

See [docs/building.md](docs/building.md) for detailed platform-specific instructions (Linux, macOS, Windows, WSL2).

### WSL2

Auto-detected. The GUI forces Mesa software rendering (`LIBGL_ALWAYS_SOFTWARE=1`, `GALLIUM_DRIVER=llvmpipe`) and X11 mode — no configuration needed.

---

## Usage

### GUI

```bash
newc                         # open GUI (auto-detects project if run from project root)
newc gui                     # explicit GUI open
newc gui ~/projects/myapp    # open GUI directly to a project
```

### CLI

```bash
newc new <name>              # scaffold a new project in the current directory
newc new <name> --git        # scaffold + initialise git repository
newc new <name> --template calculator  # scaffold + seed main.c from a template

newc add <module>            # add a new module (src/<module>.c + include/<module>.h)
newc remove                  # interactively remove a module
newc list                    # list all modules in the current project
newc sync [module]           # regenerate .h prototypes from .c definitions
newc check                   # list functions unreachable from main() (BFS)
newc tidy                    # remove unreachable functions (with confirmation)

newc stats                   # print function count and LOC per module
newc funcs [module]          # list function signatures (optional module filter)
newc search <query>          # search all .c and .h files for a string

newc update                  # download and install the latest release
newc update --check          # print whether a newer version is available
```

All CLI commands except `new` and `gui` must be run from a project root (directory containing `src/`, `include/`, and `Makefile`).

---

## Project Structure

A `newc` project looks like this:

```
myapp/
├── src/
│   ├── main.c
│   ├── input.c
│   └── math.c
├── include/
│   ├── input.h
│   └── math.h
├── build/
├── Makefile
├── .newc_meta.toml     # course, assignment, due date, marks
├── .newc_builds.json   # build history (last 100 records)
└── .gitignore          # generated if --git flag used
```

---

## Configuration

Config file: `~/.config/newc/config.toml`

```toml
terminal = "foot"           # terminal emulator for "Open in Editor"
editor = "nvim"             # text editor
theme = "dark"              # "dark" or "light"
clang_format_style = "file" # clang-format style (file/LLVM/Google/Chromium/GNU/Microsoft)
scan_dirs = ["~/projects"]  # directories scanned for existing projects on startup

[[workspaces]]
name = "CS101"
paths = ["/home/user/projects/assignment1", "/home/user/projects/assignment2"]
```

### User function library

Custom functions live in `~/.config/newc/functions/<module>.toml`:

```toml
module = "utils"

[[functions]]
name = "clamp"
description = "Clamp a value between min and max"
signature = "int clamp(int value, int min, int max)"
header_code = "int clamp(int value, int min, int max);"
impl_code = """
int clamp(int value, int min, int max) {
    if (value < min) return min;
    if (value > max) return max;
    return value;
}"""
tags = ["math", "utility"]
requires = []
```

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+P` | Quick search (projects and functions) |
| `Ctrl+Z` | Undo (Composer) |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Redo (Composer) |
| `Ctrl+S` | Save (notes, module editor, Makefile editor) |
| `?` | Open keyboard shortcuts panel |
| `Esc` | Close modal / cancel |
| `↑` `↓` | Navigate quick-search results |
| `Enter` | Confirm quick-search selection |

---

## Architecture

The project is a Cargo workspace with two crates:

```
newc-rs/
├── newc-core/          # pure logic, no UI dependencies
│   └── src/
│       ├── analysis.rs     # BFS dead-code reachability
│       ├── build_history.rs# per-project build log
│       ├── config.rs       # AppConfig (workspaces, theme, editor…)
│       ├── diag.rs         # compiler diagnostic parser
│       ├── export.rs       # ZIP export
│       ├── function_lib.rs # function library (load/save/groups)
│       ├── git.rs          # git operations wrapper
│       ├── grep.rs         # project-wide file search
│       ├── header.rs       # .h file parsing
│       ├── lint.rs         # static C linter (9 rules)
│       ├── main_builder.rs # MainBlock enum + code generation
│       ├── meta.rs         # project metadata (.newc_meta.toml)
│       ├── module.rs       # module add/remove
│       ├── notes.rs        # project notes
│       ├── project.rs      # Project struct + discovery
│       ├── project_template.rs # built-in templates
│       ├── report.rs       # Markdown report generation
│       ├── scaffold.rs     # project creation
│       ├── stats.rs        # LOC + function count metrics
│       ├── sync.rs         # .h/.c prototype sync
│       ├── templates.rs    # C reference data
│       └── user_template.rs# user-saved templates
│
└── newc/               # GUI + CLI binary
    └── src/
        ├── main.rs         # entry point, WSL2 detection
        ├── app.rs          # eframe App, event loop, view dispatch
        ├── state.rs        # AppState, View enum, persistence helpers
        ├── cli.rs          # clap CLI commands
        ├── build_runner.rs # async build via crossbeam-channel
        ├── highlight.rs    # C syntax highlighter (egui LayoutJob)
        └── views/
            ├── home.rs         # project browser + workspace tabs
            ├── create.rs       # new project form
            ├── project.rs      # project detail (modules, build, tools)
            ├── module_detail.rs# function editor + linter + call tree
            ├── header_editor.rs# .h editor with struct builder
            ├── main_builder.rs # visual main() composer
            ├── library.rs      # function library browser
            ├── git_panel.rs    # git status/stage/diff/commit/push/pull
            ├── build_history.rs# build log table
            ├── health.rs       # project health dashboard
            ├── project_search.rs # project-wide search results
            ├── usage_tracker.rs# library function usage per file
            ├── settings.rs     # app configuration form
            ├── stats.rs        # project statistics view
            ├── cref.rs         # C standard library reference
            ├── snippets.rs     # C code snippet palette
            ├── shortcuts.rs    # keyboard shortcuts modal
            └── …
```

**Key design decisions:**
- `newc-core` has zero GUI dependencies — it can be used as a library or in CLI-only contexts
- All persistent data uses TOML (config, function library, workspaces, known projects, metadata) or JSON (build history)
- The GUI uses [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) with the `glow` backend for WSL2 compatibility
- Build output is streamed via `crossbeam-channel` from a background thread

---

## Dependencies

| Crate | Purpose |
|---|---|
| `egui` / `eframe` | Immediate-mode GUI |
| `clap` | CLI argument parsing |
| `serde` + `toml` + `serde_json` | Configuration and data serialisation |
| `crossbeam-channel` | Async build output streaming |
| `rfd` | Native file dialogs |
| `dirs` | XDG config directory resolution |
| `chrono` | Timestamps in build history |
| `walkdir` | Recursive directory scanning |
| `zip` | ZIP export |
| `regex` | Prototype extraction from C source |
| `anyhow` | Error handling in CLI |
| `thiserror` | Error types in core library |

---

## License

MIT
