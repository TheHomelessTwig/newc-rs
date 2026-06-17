# newc

A Rust GUI application for scaffolding, building, and managing C projects. Built on [iced](https://github.com/iced-rs/iced) 0.14 with a multi-window MVU architecture.

![Version](https://img.shields.io/badge/version-0.6.2-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)

---

## Features

### Project Management
- **Scaffold** new C projects (`src/`, `include/`, `build/`, `Makefile` or `CMakeLists.txt`) with one command or via the GUI
- **Module system** — add/remove `.c`/`.h` pairs; auto-sync prototypes between source and header
- **Input validation** — project and module names validated as legal C identifiers at both GUI and CLI layers
- **Workspaces** — organise projects into named groups (e.g. by course or client); archive old projects
- **Project metadata** — attach course name, assignment title, due date, and marks; due-date countdown shown in project header
- **Export** — bundle a project as a ZIP, or generate a Markdown report (stats, module list, build history, notes)

### Build & Analysis
- **One-click build** — run `all`, `run`, `debug`, `release`, `strict`, `test`, `valgrind`, `analyse`, or `clean` targets from the GUI, via `make` or `cmake` depending on how the project was scaffolded
- **Live build output** — streaming stdout/stderr with timing ("Build succeeded in 1.4s")
- **Build history** — per-project log of every build (target, result, duration) stored in `.newc_builds.json`
- **Compiler diagnostics** — gcc/clang output parsed into a colour-coded table; click a row to navigate directly to the source module
- **Dead-code analysis** — BFS reachability from `main()`; unreachable functions highlighted
- **Health dashboard** — single view of dead-code count, missing includes, TODO/FIXMEs, lint warnings, header guard issues, prototype mismatches, and last build status

### Code Intelligence
- **Static linter** — 17 rules catching common C mistakes (L001–L017):
  - `L001` — `gets()` usage
  - `L002` — `strcpy()` without bounds
  - `L003` — `scanf("%s")` without width limit
  - `L004` — `printf()` with non-literal format string
  - `L005` — assignment in condition
  - `L006` — `sprintf()` without bounds
  - `L007` — `malloc`/`calloc` result not checked for `NULL`
  - `L008` — magic number literals
  - `L009` — `fopen()` with no matching `fclose()`
  - `L010` — `.h` file missing `#ifndef` header guard
  - `L011` — `free(ptr)` not followed by `ptr = NULL`
  - `L012` — `strlen()` called in loop condition
  - `L013` — `atoi()`/`atof()` usage
  - `L014` — returning address of local variable
  - `L015` — comparing pointer to non-zero integer
  - `L016` — `strtok()` not re-entrant (use `strtok_r`)
  - `L017` — `= realloc(...)` loses pointer on `NULL` return
- **Syntax highlighting** — C keywords, strings, numbers, comments, and operators colour-coded in module editor (Monokai Pro palette)
- **Function call tree** — recursive call graph (depth 5) rendered inline for any selected function
- **Project-wide search** — regex/case-insensitive grep across all `.c` and `.h` files; click result to navigate to module at line
- **Refactoring** — rename a function across the entire project, or move a function between modules with automatic header re-sync
- **Report generation** — Markdown report (stats, module list, build history, notes) from the project detail screen
- **Ctrl+P quick search** — fuzzy overlay searching functions and projects; click function to jump to C Reference entry

### Function Library
- **Built-in library** — 9 modules with 100+ functions: `input`, `math`, `display`, `array`, `algorithms`, `strings`, `linked_list`, `files`, `test_utils`
- **Structured parameter editor** — build function signatures field-by-field
- **Import from `.c`** — extract functions from any existing source file
- **Favourites** — star/unstar functions; "★ Starred" filter group
- **Custom groups** — create, rename, and delete function groups
- **User overrides** — stored in `~/.config/newc/functions/` as TOML files
- **Usage tracker** — per-project view showing which library functions are used in which source files
- **Pop-out window** — Library can be detached into a floating OS window (⊞ button in header)

### main() Composer
A visual block-based builder for `src/main.c`:
- Block types: **Variable declaration**, **Function call**, **Comment**, **Raw C**, **Blank line**, **If block**, **While loop**, **For loop**
- Undo/redo (50-level history, Ctrl+Z/Y)
- Live preview of generated `src/main.c`
- Write to file with one click
- Drag-to-reorder blocks
- **Flowchart view** — automatically generates a proper branching flowchart (YES/NO branches side-by-side, loop-back arrows for while/for)

### Git Integration
- Status view: branch, staged/unstaged/untracked file counts
- Per-file staging — stage or unstage individual files
- Unified diff view with colour-coded +/- lines
- Branch management — switch and create branches
- Commit, push, and pull from the GUI

### Code Snippets
47 built-in C patterns across 11 categories. Available as a panel or detached OS window (⊞ button in header):

| Category | Examples |
|---|---|
| Loops | for, while, do-while |
| Conditionals | if/else chain, switch/case |
| Structs & Enums | struct definition, enum, typedef |
| Memory | malloc/free, realloc, calloc |
| Files | fopen/fclose, fgets loop, binary fread/fwrite |
| Pointers | pointer arithmetic, double pointer, function pointer |
| Strings | safe copy, string scan, string comparison |
| Recursion | factorial, fibonacci |
| Error Handling | errno/perror, NULL guard, exit on error |
| Preprocessor | include guard, macro with args, conditional compile |
| Misc | argc/argv main, qsort, memset |

### Templates
11 built-in project templates: Calculator, Array Processor, Grade Manager, Menu-Driven App, File Parser, Linked List, Student Records, Recursion, CLI Arguments, State Machine, Unit Test Runner. Create custom templates by saving any project's structure.

### C Reference
Searchable reference for ~70 C standard library functions across `stdio.h`, `stdlib.h`, `string.h`, `math.h`, and `time.h`. Can be detached into a pop-out OS window (⊞ button in header). Jump directly to any function via Ctrl+P quick search.

### clang-format Integration
Format any function using clang-format. Configurable style (file, LLVM, Google, Chromium, GNU, Microsoft) in Settings.

### Per-project Settings
Override editor, terminal, and clang-format style on a per-project basis via `.newc_config.toml` in the project root. Editable from the Settings screen when a project is open.

---

## Documentation

| Document | Description |
|---|---|
| [Architecture](docs/architecture.md) | System design, module map, key types, event loop, data flow |
| [Data Formats](docs/data-formats.md) | All TOML/JSON schemas with annotated examples |
| [Building](docs/building.md) | Platform-specific build instructions (Linux, macOS, Windows, WSL2) |
| [Contributing](docs/contributing.md) | Dev setup, adding features, lint rules, templates, commit style |
| [TODOs](TODOs.md) | Known gaps and unimplemented features |

---

## Installation

### Pre-built binaries (no Rust required)

Download from [GitHub Releases](https://github.com/TheHomelessTwig/newc-rs/releases/latest):

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

### Build from source

Requires Rust stable (via [rustup](https://rustup.rs/)), `gcc`/`clang`, and `make` (or `cmake` if scaffolding CMake projects).

```bash
git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build --release
sudo cp target/release/newc /usr/local/bin/newc
```

See [docs/building.md](docs/building.md) for platform-specific instructions including WSL2.

---

## Usage

### GUI

```bash
newc                         # open GUI (terminal freed immediately)
newc gui                     # explicit form
newc gui ~/projects/myapp    # open directly to a project
```

`newc` spawns itself with a hidden `internal-gui` flag as a detached child, freeing the terminal immediately. On Windows, `CREATE_NO_WINDOW` prevents a console window appearing alongside the GUI.

### CLI

```bash
newc new <name>              # scaffold a new project (Makefile by default)
newc new <name> --git        # scaffold + git init
newc new <name> --template calculator  # scaffold from template
newc new <name> --build-system cmake   # scaffold with CMakeLists.txt instead of Makefile

newc add <module>            # add a new module (.c + .h)
newc remove                  # interactively remove a module
newc list                    # list all modules
newc sync [module]           # regenerate .h prototypes from .c definitions
newc check                   # list functions unreachable from main()
newc tidy                    # remove unreachable functions (with confirmation)

newc stats                   # print function count and LOC per module
newc funcs [module]          # list function signatures
newc search <query>          # search all .c and .h files

newc build [target]          # run a build target (default: all) — works for Makefile or CMakeLists.txt
newc test                    # alias for `newc build test`
```

`newc build` targets: `all`, `run`, `debug`, `release`, `strict`, `valgrind`, `analyse`, `test`, `clean`, `help`. Auto-detects Makefile vs CMakeLists.txt and runs the equivalent `make` or `cmake` invocation with the same flags either way — debug/release/strict drive `CMAKE_BUILD_TYPE`/`STRICT` for CMake projects, reconfiguring as needed.

All CLI commands except `new` and `gui` require a project root (directory containing `src/`, `include/`, and a `Makefile` or `CMakeLists.txt`). Build system is auto-detected from which file is present.

---

## Project Structure

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
├── Makefile            # or CMakeLists.txt, depending on --build-system
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
clang_format_style = "file" # clang-format style
scan_dirs = ["~/projects"]  # directories scanned for projects on startup

[[workspaces]]
name = "CS101"
paths = ["/home/user/projects/assignment1", "/home/user/projects/assignment2"]
```

### User function library

Custom functions in `~/.config/newc/functions/<module>.toml`:

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
| `Ctrl+S` | Save (notes, module editor, build file editor) |
| `?` | Open keyboard shortcuts panel |
| `Esc` | Close modal / cancel |
| `↑` `↓` | Navigate quick-search results |
| `Enter` | Confirm quick-search selection |

---

## Architecture

Cargo workspace with two crates:

```
newc-rs/
├── newc-core/          # pure logic, no UI dependencies
└── newc/               # GUI + CLI binary (iced 0.14)
```

See [docs/architecture.md](docs/architecture.md) for the full module map and design decisions.

---

## Dependencies

| Crate | Purpose |
|---|---|
| `iced` 0.14 | Retained-mode GUI (MVU, multi-window via daemon) |
| `clap` | CLI argument parsing |
| `serde` + `toml` + `serde_json` | Configuration and data serialisation |
| `rfd` | Native file dialogs |
| `dirs` | XDG config directory resolution |
| `chrono` | Timestamps in build history |
| `notify` | File system watcher |
| `tokio` | Async runtime for iced subscriptions |
| `futures` | Stream bridging for file watcher subscription |
| `ureq` | HTTP for self-update (GitHub Releases) |
| `anyhow` | Error handling |

---

## License

MIT
