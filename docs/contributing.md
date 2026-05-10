# Contributing

## Development setup

### Prerequisites

- Rust stable (`rustup toolchain install stable`)
- `cargo` (included with rustup)
- A C compiler (`gcc` or `clang`) for testing the generated projects
- `make`

### Clone and build

```bash
git clone https://github.com/your-username/newc-rs.git
cd newc-rs
cargo build
```

The debug binary lands at `target/debug/newc`.

### Run in development

```bash
cargo run                          # open GUI
cargo run -- gui ~/projects/myapp  # open to a specific project
cargo run -- stats                 # CLI stats in current directory
cargo run --bin newc-gui           # GUI-only binary (no console window)
```

---

## Project layout

```
newc-rs/
├── Cargo.toml                  # workspace manifest
├── newc-core/
│   ├── Cargo.toml
│   ├── src/
│   │   └── *.rs                # core logic modules
│   └── assets/
│       └── functions/
│           └── *.toml          # built-in function library data
└── newc/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── gui_main.rs         # GUI-only entry point (no console on Windows)
        ├── app.rs
        ├── state.rs
        ├── cli.rs
        ├── build_runner.rs
        ├── highlight.rs
        └── views/
            └── *.rs
```

---

## Adding a new core module

1. Create `newc-core/src/<name>.rs`
2. Add `pub mod <name>;` to `newc-core/src/lib.rs`
3. Add any new dependencies to `newc-core/Cargo.toml`

Keep all public functions pure where possible — take `&Path` instead of opening files internally where the caller might want to mock or redirect.

---

## Adding a new view

1. Create `newc/src/views/<name>.rs`
2. Add `pub mod <name>;` to `newc/src/views/mod.rs`
3. If the view needs a route, add a variant to `View` in `state.rs`
4. Handle the new `View` variant in `state.rs`'s `current_project()` match if it carries a `Project`
5. Add the view dispatch in `app.rs`'s `CentralPanel` match

**View patterns:**

For a view that replaces the whole central area (like git panel or build history):
```rust
// views/my_view.rs
pub fn show(ctx: &Context, state: &mut AppState) {
    let project = match &state.view {
        View::MyView(p) => p.clone(),
        _ => return,
    };
    egui::CentralPanel::default().show(ctx, |ui| {
        // …
    });
}
```

For a view that returns an action to `app.rs`:
```rust
// views/my_view.rs
pub enum MyAction { None, DoSomething(String), GoBack }

pub fn show(ui: &mut Ui, /* params */) -> MyAction {
    let mut action = MyAction::None;
    // …
    action
}
```

---

## Adding a new built-in function

Edit `newc-core/assets/functions/<module>.toml` (or create a new file for a new module). Re-build — the file is embedded via `include_str!`.

```toml
[[functions]]
name = "my_function"
module = "math"
description = "One-line description"
signature = "int my_function(int x)"
header_code = "int my_function(int x);"
impl_code = """
int my_function(int x) {
    return x * 2;
}
"""
tags = ["math"]
requires = []
```

If the function depends on another, list its name in `requires`. The library's `resolve_deps()` will automatically include dependencies when a user imports this function.

**If creating a new module TOML file**, also add it to `BUILTIN_TOML_FILES` in `newc-core/src/function_lib.rs`:

```rust
const BUILTIN_TOML_FILES: &[(&str, &str)] = &[
    // existing entries …
    ("my_module", include_str!("../../assets/functions/my_module.toml")),
];
```

Without this entry the module will compile into the binary but never be loaded.

---

## Adding a new lint rule

Edit `newc-core/src/lint.rs`. Two functions exist:
- `lint_file(content, filename)` — applies to `.c` files (L001–L009, L011–L015)
- `lint_header(content)` — applies to `.h` files (currently L010)

The simplest pattern for a new `.c` rule:

```rust
// L016: descriptive name
if line.contains("dangerous_pattern(") {
    warnings.push(LintWarning {
        line_no: lno,
        severity: LintSeverity::Warning,
        code: "L016",
        message: "Explain the problem and what to use instead".into(),
    });
}
```

Rules must:
- Operate on a single `&str` line (or a small lookahead window for multi-line patterns)
- Never panic
- Be fast enough to run every render frame without noticeable lag
- Have at least two unit tests: one that triggers the rule, one clean case that does not

Watch for false positives from function names that contain the matched substring (e.g. `fgets` contains `gets`, `snprintf` contains `printf`). Use a `prev_is_alpha` guard when needed:

```rust
let prev_is_alpha = pos > 0 && line.as_bytes()[pos - 1].is_ascii_alphabetic();
if !prev_is_alpha { /* fire rule */ }
```

---

## Adding a project template

Edit `newc-core/src/project_template.rs`:

1. Add an entry to `all_templates()`:
```rust
ProjectTemplate {
    name: "My Template",
    description: "One-line description",
    modules: &["input", "display"],
    builder: my_template_builder,
},
```

2. Implement the builder:
```rust
fn my_template_builder() -> MainBuilderState {
    let blocks = vec![
        MainBlock::FunctionCall {
            func_name: "print_header".into(),
            args: vec![r#""My App""#.into()],
            assign_to: String::new(),
            comment: String::new(),
        },
        // …
    ];
    MainBuilderState {
        blocks,
        globals: Vec::new(),
        includes: vec!["input".into(), "display".into()],
    }
}
```

---

## Adding a new default module

A default module is one the user can include at project-creation time. Adding one requires three steps:

1. **Add C file content** — add `pub fn my_module_h() -> &'static str` and `pub fn my_module_c() -> &'static str` to `newc-core/src/templates.rs`

2. **Wire into scaffold** — add a variant to `DefaultModule` in `newc-core/src/scaffold.rs` and handle it in the `match` arm that writes module files

3. **Add library TOML** — create `newc-core/assets/functions/my_module.toml` and add it to `BUILTIN_TOML_FILES` in `function_lib.rs` so it appears in the library browser

Optionally, wire the checkbox into `newc/src/views/create.rs` so users can select it in the GUI project creation form.

---

## Commit style

```
<type>: <short description>

Optional body.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

---

## Testing

### Automated tests

Run the full suite:
```bash
cargo test --workspace
```

48 tests across 5 modules (all in `newc-core` or `newc` — no display required):

| Module | Tests | Coverage |
|---|---|---|
| `lint.rs` | 30 | All 15 lint rules — trigger and clean case; false-positive regressions for L001/L004 |
| `sync.rs` | 4 | `extract_signatures` — single, multiple, empty, text accuracy |
| `module.rs` | 5 | `add_module`, `remove_module` with tempdir — files, includes, duplicates |
| `scaffold.rs` | 4 | `create_project` — dirs, module files, duplicate error, main.c includes |
| `updater.rs` | 5 | `semver_gt` — patch/minor/major bump, same, older |

CI runs `cargo test --workspace` on every push and pull request to `main` via `.github/workflows/ci.yml`.

### Manual GUI testing

For changes affecting the GUI, build and run manually:
```bash
cargo build
cargo run -- new testproj
cd testproj
cargo run -- add mymodule
cargo run -- sync
cargo run -- check
cargo run -- gui .
```

Verify: template picker, function library import, Composer write, build output, git panel.

For WSL2:
```bash
LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe cargo run
```

---

## Release process

1. Bump the version in both `Cargo.toml` files to the new version.
2. Commit: `git commit -m "chore: bump to vX.Y.Z"`
3. Tag: `git tag vX.Y.Z`
4. Push tag: `git push origin main --tags`

GitHub Actions (`.github/workflows/release.yml`) then automatically:
- Runs the test suite
- Builds binaries for Linux x86_64/aarch64, macOS ARM, and Windows x86_64
- Creates a GitHub Release with all binaries attached

Once the release is live, users on any platform can install it with:
```bash
newc update
```

---

## Dependencies policy

- `newc-core` must have zero GUI dependencies
- Prefer `std` over external crates where the implementation is small
- Platform-specific crates must be gated with `[target.'cfg(...)'.dependencies]`
- All new dependencies require a rationale comment in `Cargo.toml`
