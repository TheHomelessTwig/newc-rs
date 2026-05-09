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

---

## Adding a new lint rule

Edit `newc-core/src/lint.rs`. The simplest pattern:

```rust
// L010: descriptive name
if line.contains("dangerous_pattern(") {
    warnings.push(LintWarning {
        line_no: lno,
        severity: LintSeverity::Warning,
        code: "L010",
        message: "Explain the problem and what to use instead".into(),
    });
}
```

Rules must:
- Operate on a single `&str` line (or a small lookahead window for multi-line patterns)
- Never panic
- Be fast enough to run every render frame without noticeable lag

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

## Commit style

```
<type>: <short description>

Optional body.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

The pre-commit hook automatically bumps the patch version in both `Cargo.toml` files. Do not edit version numbers manually.

---

## Testing

### Automated tests

Run the full suite:
```bash
cargo test --workspace
```

39 tests across 5 modules (all in `newc-core` or `newc` — no display required):

| Module | Tests | Coverage |
|---|---|---|
| `lint.rs` | 18 | All 9 lint rules — trigger and clean case, including bug-fix regressions |
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

1. For patch releases: the pre-commit hook bumps the version automatically on every commit — no manual action needed.
2. For minor/major releases: edit the version in both `Cargo.toml` files manually before committing, then commit with `--no-verify` to skip the hook's auto-increment.
3. Tag: `git tag v0.x.y`
4. Push tag: `git push origin main --tags`

GitHub Actions (`.github/workflows/release.yml`) then automatically:
- Runs the test suite
- Builds binaries for Linux x86_64/aarch64, macOS Intel/ARM, and Windows x86_64
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
