# newc

A Rust GUI application for scaffolding, building, and managing C projects. Replaces a hand-rolled bash script with a fully-featured desktop tool built on [egui](https://github.com/emilk/egui).

![Version](https://img.shields.io/badge/version-0.4.0-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)

---

## Features

### Project Management
- **Scaffold** new C projects (`src/`, `include/`, `build/`, `Makefile`) with one command or via the GUI
- **Module system** — add/remove `.c`/`.h` pairs; auto-sync prototypes between source and header
- **Input validation** — project and module names are validated as legal C identifiers at both the GUI and CLI layers; invalid names are rejected with inline feedback before any files are created
- **Workspaces** — organise projects into named groups (e.g. by course or client); archive old projects
- **Project metadata** — attach course name, assignment title, due date, and marks; due-date countdown shown in the project header
- **Export** — bundle a project as a ZIP, or generate a Markdown report (stats, module list, build history, notes)
- **Recent projects** — last 5 opened shown in the sidebar for quick access

### Build & Analysis
- **One-click make** — run `all`, `run`, `debug`, `release`, `strict`, `test`, or `clean` targets from the GUI
- **Live build output** — streaming stdout/stderr, with timing ("Build succeeded in 1.4s")
- **Build history** — per-project log of every build (target, result, duration) stored in `.newc_builds.json`
- **Compiler diagnostics** — gcc/clang output parsed into a colour-coded table after each build; click a row to navigate directly to the source module and highlight the flagged line
- **Dead-code analysis** — BFS reachability from `main()`; unreachable functions highlighted in orange
- **Health dashboard** — single view of dead-code count, missing includes, TODO/FIXMEs, lint warnings, header guard issues, prototype mismatches, and last build status; mtime-based cache avoids recomputing on unchanged files

### Code Intelligence
- **Static linter** — 15 rules catching common C mistakes:
  - `L001` — `gets()` usage (unsafe; use `fgets`)
  - `L002` — `strcpy()` without bounds (use `strncpy`)
  - `L003` — `scanf("%s")` without width limit
  - `L004` — `printf()` with non-literal format string
  - `L005` — assignment in condition (`if (x = y)`)
  - `L006` — `sprintf()` without bounds (use `snprintf`)
  - `L007` — `malloc`/`calloc` result not checked for `NULL`
  - `L008` — magic number literals
  - `L009` — `fopen()` with no matching `fclose()`
  - `L010` — `.h` file missing `#ifndef` header guard
  - `L011` — `free(ptr)` not followed by `ptr = NULL` within 3 lines
  - `L012` — `strlen()` called in a loop condition (performance bug)
  - `L013` — `atoi()`/`atof()` usage (suggest `strtol`/`strtod` with error checking)
  - `L014` — returning address of a local variable
  - `L015` — comparing a pointer to a non-zero integer literal
- **Syntax highlighting** — C keywords, strings, numbers, comments, and operators colour-coded in the module editor
- **Missing include detection** — warns when a library function is called but its `.h` is not included; one-click auto-fix
- **Prototype mismatch detection** — banner shown after saving a function if the `.c` signature diverges from the `.h`; also reported as a health card
- **Function call tree** — recursive call graph (up to depth 5) rendered inline for any selected function
- **Project-wide search** — grep across all `.c` and `.h` files with highlighted match display and click-to-navigate
- **Diagnostic click-through** — clicking a compiler error/warning in the build panel navigates to the corresponding module and highlights the flagged line

### Function Library
- **Built-in library** — 9 modules with 100+ functions:
  - `input` — safe integer/float/string input helpers
  - `math` — common math utilities
  - `display` — formatted output helpers
  - `array` — sorting, searching, printing integer arrays
  - `algorithms` — binary search, bubble sort, selection sort, merge sort, etc.
  - `strings` — safe string operations (`str_copy_safe`, `str_to_upper`, `str_reverse`, `str_trim`, `str_is_palindrome`, …)
  - `linked_list` — singly-linked list operations (`list_insert`, `list_append`, `list_find`, `list_reverse`, …)
  - `files` — safe file I/O helpers (`file_open_safe`, `file_count_lines`, `file_copy`, …)
  - `test_utils` — unit test harness (`assert_int_eq`, `assert_str_eq`, `assert_double_near`, `test_run`, `print_test_summary`)
- **Auto-detect requirements** — "Detect" button scans implementation code and auto-fills `requires` (stdlib headers + known library function dependencies)
- **Structured parameter editor** — build function signatures field-by-field (name, return type, parameters); the signature field is auto-generated and can be overridden
- **Import from `.c`** — extract functions from any existing source file and add to the library
- **Favourites** — star/unstar functions; "★ Starred" filter group in the sidebar
- **Custom groups** — create, rename, and delete function groups
- **User overrides** — stored in `~/.config/newc/functions/` as TOML files
- **Usage tracker** — per-project view showing which library functions are used in which source files
- **Pop-out window** — Library can be undocked into a floating OS window so it can be tiled alongside the main application

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
47 built-in C patterns across 11 categories, available as a panel in the main view or as a pop-out OS window:

| Category | Examples |
|---|---|
| **Loops** | for, while, do-while |
| **Conditionals** | if/else chain, switch/case |
| **Structs & Enums** | struct definition, enum, typedef |
| **Memory** | malloc/free, realloc, calloc |
| **Files** | fopen/fclose, fgets loop, binary fread/fwrite |
| **Pointers** | pointer arithmetic, double pointer, function pointer |
| **Strings** | safe copy, string scan, string comparison |
| **Recursion** | factorial, fibonacci |
| **Error Handling** | errno/perror, NULL guard, exit on error |
| **Preprocessor** | include guard, macro with args, conditional compile |
| **Misc** | argc/argv main, qsort, memset |

### Templates
11 built-in project templates with pre-seeded `main()`:
- **Calculator** — arithmetic operations with input validation
- **Array Processor** — fill, sort, and print integer arrays
- **Grade Manager** — student grade entry and statistics
- **Menu-Driven App** — switch-based interactive menu loop
- **File Parser** — read and process a text file line by line
- **Linked List** — singly-linked list demo with insert/print/free
- **Student Records** — struct array with search and display
- **Recursion** — fibonacci (iterative + recursive) and factorial, with base-case commentary
- **CLI Arguments** — `argc`/`argv` parsing with `--help` flag and option loop
- **State Machine** — enum-based state machine with switch dispatch (vending-machine example)
- **Unit Test Runner** — `main()` calls `run_all_tests()`; demonstrates assert pattern and test registration

Create custom templates by saving any project's structure.

### C Reference
Searchable reference for ~70 C standard library functions across `stdio.h`, `stdlib.h`, `string.h`, `math.h`, and `time.h` — with signatures, descriptions, and usage examples. Can be opened as a pop-out OS window.

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
| macOS Apple Silicon | `newc-aarch64-macos` |
| Windows x86_64 | `newc-x86_64-windows.exe` |

```bash
# Linux x86_64
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
newc                         # open GUI (terminal is freed immediately)
newc gui                     # same, explicit form
newc gui ~/projects/myapp    # open GUI directly to a project
```

`newc` spawns itself with a hidden `--internal-gui` flag as a detached child process, freeing the terminal immediately. On Windows, `CREATE_NO_WINDOW` is set on the child so no console window appears alongside the GUI.

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

newc test                    # run `make test` in the current project

newc update                  # download and install the latest release (newc + newc-gui)
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

When the `test_utils` module is included, the Makefile gains a `test:` target that compiles and runs the test binary.

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
│       ├── function_lib.rs # function library (load/save/groups/detect_requires)
│       ├── git.rs          # git operations wrapper
│       ├── grep.rs         # project-wide file search
│       ├── header.rs       # .h file parsing
│       ├── lint.rs         # static C linter (15 rules)
│       ├── main_builder.rs # MainBlock enum + code generation
│       ├── meta.rs         # project metadata (.newc_meta.toml)
│       ├── module.rs       # module add/remove + C identifier validation
│       ├── notes.rs        # project notes
│       ├── project.rs      # Project struct + discovery
│       ├── project_template.rs # 11 built-in templates
│       ├── report.rs       # Markdown report generation
│       ├── scaffold.rs     # project creation + DefaultModule enum
│       ├── stats.rs        # LOC + function count metrics
│       ├── sync.rs         # .h/.c prototype sync
│       ├── templates.rs    # C file content for all default modules
│       └── user_template.rs# user-saved templates
│
└── newc/               # GUI + CLI binary
    └── src/
        ├── main.rs         # entry point: CLI dispatch + self-spawn for detached GUI
        ├── app.rs          # eframe App, SharedTools (Arc<Mutex>), event loop
        ├── state.rs        # AppState, View enum, persistence helpers
        ├── cli.rs          # clap CLI commands (including `test` subcommand)
        ├── build_runner.rs # async build via crossbeam-channel
        ├── highlight.rs    # C syntax highlighter (egui LayoutJob)
        └── views/
            ├── home.rs         # project browser + workspace tabs
            ├── create.rs       # new project form with inline validation
            ├── project.rs      # project detail (modules, build, tools)
            ├── module_detail.rs# function editor + linter + call tree + line highlight
            ├── header_editor.rs# .h editor with struct builder
            ├── main_builder.rs # visual main() composer
            ├── library.rs      # function library with structured param editor
            ├── git_panel.rs    # git status/stage/diff/commit/push/pull
            ├── build_history.rs# build log table
            ├── health.rs       # health dashboard (mtime cache, new cards)
            ├── project_search.rs # project-wide search results
            ├── usage_tracker.rs# library function usage per file
            ├── settings.rs     # app configuration form
            ├── stats.rs        # project statistics view
            ├── cref.rs         # C standard library reference (pop-out capable)
            ├── snippets.rs     # C code snippet palette (pop-out capable)
            ├── build_panel.rs  # streaming build output + clickable diagnostics
            ├── shortcuts.rs    # keyboard shortcuts modal
            └── …
```

**Key design decisions:**
- `newc-core` has zero GUI dependencies — it can be used as a library or in CLI-only contexts
- All persistent data uses TOML (config, function library, workspaces, known projects, metadata) or JSON (build history)
- The GUI uses [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) with the `glow` backend for WSL2 compatibility
- Build output is streamed via `crossbeam-channel` from a background thread
- Floating windows (Library, C Reference, Snippets) use `Arc<Mutex<SharedTools>>` to share state with `'static` viewport closures required by `show_viewport_deferred`

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
