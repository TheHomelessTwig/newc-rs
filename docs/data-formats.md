# Data Formats

All persistent data uses TOML or JSON. No binary formats, no databases.

---

## `~/.config/newc/config.toml`

Application configuration. Created on first save from Settings.

```toml
terminal = "foot"
editor = "nvim"
theme = "dark"                # "dark" | "light"
clang_format_style = "file"   # passed to clang-format -style=
code_font_size = 13.0         # monospace size for code views and editors

scan_dirs = [
    "~/projects",
    "~/uni",
]

# Each workspace groups a set of project paths
[[workspaces]]
name = "CS101"
paths = [
    "/home/user/projects/assignment1",
    "/home/user/projects/assignment2",
]

[[workspaces]]
name = "__archived__"         # reserved name for archived projects
paths = [
    "/home/user/projects/old-project",
]
```

**Defaults** (used if the file is absent or a key is missing):

| Key | Default |
|---|---|
| `terminal` | `foot` / `kitty` / `alacritty` / `wezterm` / `xterm` (first found), `Terminal` on macOS, `wt` on Windows |
| `editor` | `nvim` / `vim` / `nano` / `code` (first found) |
| `theme` | `"dark"` |
| `clang_format_style` | `"file"` |
| `scan_dirs` | `["~/projects"]` |
| `workspaces` | `[]` |
| `code_font_size` | `13.0` |

---

## `<project>/.newc_config.toml`

Per-project overrides of the global config, plus named build profiles. Any
key left unset falls back to the global `config.toml` value.

```toml
editor = "code"
terminal = "kitty"
clang_format_style = "LLVM"

[[build_profiles]]
name = "asan"
cflags = "-fsanitize=address,undefined -g"

[[build_profiles]]
name = "tiny"
cflags = "-Os -DNDEBUG"
```

Selected via a dropdown in the build panel; the chosen profile's `cflags` is
passed as `make <target> EXTRA_CFLAGS="..."` (Make projects only).

---

## `~/.config/newc/projects.toml`

Ordered list of all known project paths. Updated whenever a project is opened or removed.

```toml
projects = [
    "/home/user/projects/calculator",
    "/home/user/projects/linked-list",
]
```

---

## `~/.config/newc/recents.toml`

Last 5 opened project paths (most recent first).

```toml
recent = [
    "/home/user/projects/calculator",
    "/home/user/projects/linked-list",
]
```

---

## `~/.config/newc/groups.toml`

User-defined function group names. Built-in groups (`input`, `math`, `display`, `array`, `algorithms`) are not stored here.

```toml
[[groups]]
name = "utils"
description = "General utility functions"

[[groups]]
name = "sorting"
description = "Custom sorting algorithms"
```

---

## `~/.config/newc/functions/<module>.toml`

User-defined or overridden function library entries. One file per module.

```toml
module = "utils"

[[functions]]
name = "clamp"
module = "utils"
description = "Clamp a value between min and max"
signature = "int clamp(int value, int min, int max)"
header_code = "int clamp(int value, int min, int max);"
impl_code = """
int clamp(int value, int min, int max) {
    if (value < min) return min;
    if (value > max) return max;
    return value;
}
"""
requires = []           # names of other functions this one depends on
tags = ["math", "utility"]
notes = "Safe to use with negative values."
starred = false
```

**Fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | String | Yes | Unique function name (matches C function name) |
| `module` | String | Yes | Group name; must match the file stem |
| `description` | String | Yes | One-line description shown in the library |
| `signature` | String | Yes | Full C signature without trailing `;` |
| `header_code` | String | Yes | Text injected into `.h` (usually signature + `;`) |
| `impl_code` | String | Yes | Full function definition injected into `.c` |
| `requires` | `[String]` | No | Other function names this function calls |
| `tags` | `[String]` | No | Search tags |
| `notes` | String | No | Longer description shown in detail pane |
| `starred` | bool | No | Favourite flag (default `false`) |

When `requires` is populated, `FunctionLibrary::resolve_deps()` ensures all dependencies are included when importing a function into a module.

---

## `~/.config/newc/templates/<name>.toml`

User-saved project templates (created via "Save as Template" in the GUI).

```toml
name = "My Template"
description = "Custom project with specialised modules"
modules = ["input", "display", "utils"]

[[globals]]
type_name = "int"
name = "MAX_SIZE"
init = "100"
is_array = false
array_size = ""
is_static = false

[[blocks]]
type = "FunctionCall"
func_name = "print_header"
args = ['"My App"']
assign_to = ""
comment = ""

[[blocks]]
type = "BlankLine"

[[blocks]]
type = "VarDecl"
type_name = "int"
name = "n"
init = ""
is_array = false
array_size = ""
```

The `blocks` array serialises `MainBlock` enum variants. The `type` field discriminates the variant; remaining fields match the variant's struct fields.

---

## `<project>/.newc_meta.toml`

Per-project academic metadata.

```toml
course = "CS101"
assignment = "Assignment 2 — Linked Lists"
due_date = "2025-05-30"    # YYYY-MM-DD; empty string if not set
max_marks = 100
received_marks = 87
notes = "Focus on memory management and pointer safety."
```

All fields are optional (default to empty string or absent).

---

## `<project>/.newc_builds.json`

Build history. Appended after every build; capped at 100 records (oldest removed).

```json
[
  {
    "timestamp": "2025-05-09 14:32:01",
    "target": "all",
    "exit_code": 0,
    "duration_ms": 1423
  },
  {
    "timestamp": "2025-05-09 14:35:47",
    "target": "run",
    "exit_code": 0,
    "duration_ms": 312
  }
]
```

| Field | Type | Description |
|---|---|---|
| `timestamp` | String | `YYYY-MM-DD HH:MM:SS` local time |
| `target` | String | Make target (`all`, `run`, `debug`, `release`, `strict`, `clean`) |
| `exit_code` | int or null | Process exit code; `null` if killed |
| `duration_ms` | u64 | Wall-clock build time in milliseconds |

---

## Built-in function library (embedded TOML)

The built-in library is compiled into the binary using `include_str!`. Source files are in `newc-core/assets/functions/`:

```
assets/functions/
├── input.toml        # prompt_for_int, prompt_for_double, prompt_for_char, prompt_for_string
├── math.toml         # clamp, power, is_prime, gcd, factorial, …
├── display.toml      # print_header, print_separator, press_enter_to_exit, print_table, …
├── array.toml        # array_input_int, array_sort_asc, array_sum, array_print, array_average_double, …
└── algorithms.toml   # bubble_sort, selection_sort, binary_search, merge_sort, …
```

These follow the same TOML schema as user functions. User-defined functions in `~/.config/newc/functions/` can **override** any built-in by using the same `name` field — the user version replaces the built-in at load time.

---

## C source file conventions

`newc` reads and writes C source files with these conventions:

### Header guard

Generated headers use `#pragma once`:

```c
#pragma once

/* SYNC_IGNORE_START */
// Structs, typedefs, and macros go here — sync will not touch this block
/* SYNC_IGNORE_END */

// Auto-generated prototypes below
int my_function(int x, int y);
```

The `SYNC_IGNORE_START` / `SYNC_IGNORE_END` markers protect user content from being overwritten by `newc sync`.

### Generated `main.c`

The Composer generates `main.c` with this skeleton:

```c
/*
 * main.c
 * Author: <author>
 * Date: <DD/MM/YYYY>
 */

#include <stdio.h>
#include <stdlib.h>
#include "input.h"
#include "display.h"

// Global variable declarations
int MAX_SIZE = 100;

int main(void) {
    // … generated blocks …
    return 0;
}
```

### Build file (Makefile or CMakeLists.txt)

`newc new --build-system make|cmake` (default `make`) controls which file is generated. The
project's build system is auto-detected afterwards from whichever file exists at the root —
no separate config flag is stored. Both expose the same targets, with the CMake side
implemented as `add_custom_target` wrappers around an `add_executable(main ...)`:

| Target | Description |
|---|---|
| `all` | Compile all `.c` files to `build/<project>` (or `cmake --build build`) |
| `run` | Build then execute |
| `debug` | Build with `-g -fsanitize=address,undefined` (CMake: configure `-DCMAKE_BUILD_TYPE=Debug`) |
| `release` | Build with `-O2` (CMake: configure `-DCMAKE_BUILD_TYPE=Release`, the default) |
| `strict` | Build with all warnings as errors (CMake: configure `-DSTRICT=ON`, clean rebuild) |
| `valgrind` | Run the built executable under Valgrind |
| `valgrind-xml` | Same as `valgrind`, writes structured XML to `vg.xml` for the GUI to parse |
| `analyse` | Run clang static analysis (syntax-only) |
| `cppcheck` | Run cppcheck static analysis (requires `cppcheck` installed) |
| `coverage` | Build with gcov instrumentation, run, and emit `.gcov` reports (CMake: configure `-DCOVERAGE=ON`) |
| `test` | Run the built executable (only present if the `test_utils` or `unity` module was included at scaffold time) |
| `clean` | Remove `build/` directory (or `cmake --build build --target clean`) |

A CMake project also gets a `CMakePresets.json` (debug/release/strict/coverage) mirroring the same `-D` flags; the build runner uses `cmake --preset <name>` instead of hand-built `-D` args when that file is present.
