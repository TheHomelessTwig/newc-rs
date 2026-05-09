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

No automated test suite currently exists. Manual testing procedure:

1. Build: `cargo build`
2. Create a test project: `cargo run -- new testproj && cd testproj`
3. Add a module: `cargo run -- add mymodule`
4. Check sync: `cargo run -- sync`
5. Check dead-code: `cargo run -- check`
6. Open GUI: `cargo run -- gui .`
7. Verify: template picker, function library import, Composer write, build output, git panel

For platform testing on Linux, use the WSL2 path:
```bash
LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe cargo run
```

---

## Release process

1. Bump major/minor version manually in both `Cargo.toml` files before the release commit
   - The hook bumps **patch** automatically; for minor/major bumps, edit manually first
2. Tag: `git tag v0.x.y`
3. Push: `git push origin main --tags`
4. Build release binary: `cargo build --release`

---

## Dependencies policy

- `newc-core` must have zero GUI dependencies
- Prefer `std` over external crates where the implementation is small
- Platform-specific crates must be gated with `[target.'cfg(...)'.dependencies]`
- All new dependencies require a rationale comment in `Cargo.toml`
