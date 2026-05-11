# Contributing

## Development setup

### Prerequisites

- Rust stable (`rustup toolchain install stable`)
- `cargo` (included with rustup)
- A C compiler (`gcc` or `clang`) for testing generated projects
- `make`
- Vulkan or Mesa drivers (for running the GUI locally — see [building.md](building.md))

### Clone and build

```bash
git clone https://github.com/TheHomelessTwig/newc-rs.git
cd newc-rs
cargo build
```

Debug binary at `target/debug/newc`.

### Run in development

```bash
cargo run                          # open GUI
cargo run -- gui ~/projects/myapp  # open to a specific project
cargo run -- stats                 # CLI stats in current directory
```

On WSL2 without GPU passthrough:

```bash
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json cargo run
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
        ├── main.rs             # entry point (CLI dispatch + GUI spawn)
        ├── app.rs              # NewcApp — update(), view_for_window(), subscriptions
        ├── state.rs            # AppState, View enum, Message enum
        ├── cli.rs              # clap CLI commands
        ├── build_runner.rs     # async build via channel
        ├── highlight.rs        # C syntax tokeniser
        ├── theme.rs            # color constants + style functions
        └── views/
            └── *.rs            # one file per view
```

---

## Adding a new view

1. Create `newc/src/views/<name>.rs` with a `pub fn view(state: &AppState, ...) -> Element<'_, Message>` function
2. Add `pub mod <name>;` to `newc/src/views/mod.rs`
3. Add a variant to `View` in `state.rs`
4. Handle the new variant in `state.rs`'s `current_project()` match if it carries a `Project`
5. Add dispatch in `app.rs`'s `central_panel()` match

**View pattern — iced MVU style:**

```rust
// views/my_view.rs
use iced::Element;
use crate::state::{AppState, Message};
use crate::theme as th;

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back").size(12))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        th::heading("My View"),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    column![header, th::separator(), /* content */]
        .spacing(8)
        .padding(12)
        .into()
}
```

Views are pure functions — no I/O, no mutation. Emit `Message` values; `update()` in `app.rs` handles side effects.

**Adding a new Message:**

```rust
// state.rs — add to Message enum
MyAction(String),

// app.rs — add to update() match
Message::MyAction(s) => {
    // perform side effect
    Task::none()
}
```

---

## Styling

Use `crate::theme as th` — never hardcode colors or create inline style closures in views.

```rust
// Colors
th::color::ACCENT    // #FF6188 coral
th::color::GREEN     // #A9DC76
th::color::CYAN      // #78DCE8
th::color::TEXT      // #FCFCFA
th::color::TEXT_HINT // #656266

// Container styles (fn(&iced::Theme) -> container::Style)
th::panel_style      // sidebar panel
th::card_style       // rounded card
th::section_style    // inner group
th::deep_style       // darkest background

// Button styles (fn(&iced::Theme, button::Status) -> button::Style)
th::btn_primary      // coral fill
th::btn_secondary    // subtle border
th::btn_ghost        // no border
th::btn_danger       // red tint
th::btn_nav_active   // active nav tab
th::btn_nav_inactive // inactive nav tab

// Typography (return Text<'static>)
th::heading("Title")       // size 20
th::subheading("Sub")      // size 15, dim
th::section_title("Group") // size 13, cyan
th::label_text("Label")    // size 12
th::hint_text("Hint")      // size 11, hint color

// Helpers
th::separator()            // 1px horizontal divider
```

---

## Adding a new built-in function

Edit `newc-core/assets/functions/<module>.toml`:

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

Re-build — the file is embedded via `include_str!`. If creating a new module TOML file, also add it to `BUILTIN_TOML_FILES` in `newc-core/src/function_lib.rs`.

---

## Adding a new lint rule

Edit `newc-core/src/lint.rs`. Two functions:
- `lint_file(content)` — `.c` files (L001–L009, L011–L015)
- `lint_header(content)` — `.h` files (L010)

```rust
// L016: example rule
if line.contains("dangerous_pattern(") {
    warnings.push(LintWarning {
        line_no: lno,
        severity: LintSeverity::Warning,
        code: "L016",
        message: "Explain the problem and the alternative".into(),
    });
}
```

Rules must: operate per-line, never panic, run fast enough to apply on every view render, have unit tests for trigger and clean cases.

Watch for false positives — use a `prev_is_alpha` guard to distinguish `fgets` from `gets`:

```rust
let prev_is_alpha = pos > 0 && line.as_bytes()[pos - 1].is_ascii_alphabetic();
if !prev_is_alpha { /* fire rule */ }
```

---

## Adding a project template

Edit `newc-core/src/project_template.rs`:

1. Add entry to `all_templates()`:
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
            func_name: "my_func".into(),
            args: vec![],
            assign_to: String::new(),
            comment: String::new(),
        },
    ];
    MainBuilderState { blocks, globals: Vec::new(), includes: vec!["input".into()] }
}
```

---

## Commit style

```
<type>: <short description>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

---

## Testing

```bash
cargo test --workspace
```

Tests live in `newc-core` (lint, sync, module, scaffold, updater). No display required.

For GUI changes, build and run manually:

```bash
cargo build
cargo run -- new testproj && cd testproj
cargo run -- add mymodule && cargo run -- sync
cargo run -- gui .
```

---

## Release process

1. Bump version in `newc/Cargo.toml` and `newc-core/Cargo.toml`
2. Commit: `git commit -m "chore: bump to vX.Y.Z"`
3. Tag: `git tag vX.Y.Z`
4. Push: `git push origin main --tags`

GitHub Actions then builds binaries for all platforms and creates a GitHub Release.

---

## Dependencies policy

- `newc-core` must have zero GUI dependencies
- Prefer `std` over external crates for small implementations
- Platform-specific crates gated with `[target.'cfg(...)'.dependencies]`
- All new dependencies require a rationale comment in `Cargo.toml`
