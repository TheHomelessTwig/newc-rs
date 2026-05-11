# newc TODOs — iced 0.14 Port

Comprehensive list of known gaps, bugs, and unimplemented features as of v0.5.0.
Priority: P0 = blocking/visible bug, P1 = stub handler (feature does nothing), P2 = theme gap, P3 = feature completeness.

---

## P0 — Bugs (visible/functional issues)

### Syntax Highlighting Not Rendering
- **Location:** `newc/src/views/module_detail.rs` → `code_view()`
- **Status:** Tokenizer (`highlight_c()`) is wired and produces colored spans. Rendering uses `row![]` of individual `text().color()` widgets per line with `spacing(0)`. Colors may be invisible in WSL2 software renderer or tokens may not align correctly.
- **Fix options:**
  1. Switch to iced's built-in `highlighter` feature (already in `Cargo.toml`) with a `text_editor` widget for code display — provides proper glyph-aligned rendering.
  2. Debug by testing on native GPU to rule out WSL2 rendering artifact.
  3. As fallback: render each line as a single `text()` with no coloring until highlighting is debugged.
- **Also missing:** highlighting is only in module_detail. Not applied in: `header_editor.rs` (plain `text_editor`), `project_search.rs` (match context snippets).

### Module Editor — Edit Mode is Read-Only
- **Location:** `newc/src/views/module_detail.rs:195`
- **Status:** Edit mode renders `text(mds.edit_buf.as_str())` — a static display widget. Cannot type to modify the function body.
- **Fix:** Replace with `text_editor(&content).on_action(Message::ModuleEditBuf)` backed by `text_editor::Content`. State already has `edit_buf: String`; needs to become `edit_content: text_editor::Content` (like `notes_content` and `makefile_content` already are).

### DiagJumpTo Line Highlight Not Applied
- **Location:** `newc/src/state.rs` → `ModuleDetailState.highlight_line`; `newc/src/app.rs:710`
- **Status:** `highlight_line` is set when a compiler diagnostic is clicked. `code_view()` never reads it — no visual scroll or highlight occurs.
- **Fix:** Pass `highlight_line` into `code_view()` and either scroll to that line or render a highlighted background on the matching row.

### Shortcuts Panel Not an Overlay
- **Location:** `newc/src/views/shortcuts.rs`
- **Status:** The shortcuts panel renders as a plain column in the content area — not a floating modal. It replaces content rather than overlaying it.
- **Fix:** Integrate with `widget::Stack` in `view_main()` so it overlays the current view with a semi-transparent backdrop, matching the design of quick_search.

### Quick Search Overlay Integration Unverified
- **Location:** `newc/src/views/quick_search.rs`, `newc/src/app.rs`
- **Status:** Quick search UI exists but its Stack overlay positioning needs verification — it may render inline rather than as a centered floating card.
- **Fix:** Confirm it appears over the main content using `Stack` with pointer-passthrough on the base layer.

---

## P1 — Stub Handlers (features that do nothing)

### `ModuleClangFormat` — Empty
- **Location:** `newc/src/app.rs:954`
- **Handler:** `Message::ModuleClangFormat => { /* TODO */ }`
- **Fix:** Read current module source, write to temp file, run `clang-format -style=<cfg> -i <path>`, reload content. Use `state.config.clang_format_style`.

### `ModuleDeleteFunc` — Empty
- **Location:** `newc/src/app.rs:936`
- **Handler:** `Message::ModuleDeleteFunc(_name) => { /* comment only */ }`
- **Fix:** Show confirmation modal (reuse `confirm_remove_module` pattern), then splice the function body from the `.c` file and re-sync the header.

### `ModuleAddFromLibrary` — Empty
- **Location:** `newc/src/app.rs:947`
- **Handler:** `Message::ModuleAddFromLibrary => { /* TODO */ }`
- **Fix:** Open `views::function_picker` to let the user select a library function, then insert its `impl_code` into the module's `.c` file at the end.
- **Also:** The "From Library" button in `module_detail.rs:108` fires `Message::None` — wire to `Message::ModuleAddFromLibrary`.

### `UpdateCheck` / `UpdateInstall` — Stub
- **Location:** `newc/src/app.rs:704`
- **Handler:** Empty match arm with comment.
- **Fix:** Port the updater logic from the egui version using `ureq` (already in Cargo.toml) — check GitHub Releases API, compare version, download binary.

### Git Panel "Init Repository" — `Message::None`
- **Location:** `newc/src/views/git_panel.rs:24`
- **Fix:** Wire to a new `Message::GitInit` that calls `git init` via `newc_core::git` in the project root.

### `MoveToWorkspace` — No Handler
- **Location:** `newc/src/state.rs:251`, `newc/src/app.rs`
- **Status:** Message variant exists but no `update()` arm handles it.
- **Fix:** Add handler that moves the project path from its current workspace into the target workspace in `state.config.workspaces`, then saves config.

---

## P2 — Theme / Style Gaps

Views still using hardcoded `Color::from_rgb(...)` or unstyled default buttons. Apply `th::btn_primary`, `th::btn_secondary`, `th::btn_ghost`, `th::btn_danger`, `th::section_style`, `th::separator()`, and `th::color::*` constants.

| View | Issue |
|---|---|
| `add_module.rs` | Unstyled buttons, no separator |
| `build_history.rs` | Hardcoded colors in row cells |
| `build_panel.rs` | Hardcoded colors for stdout/stderr/done lines |
| `create.rs` | Unstyled form, no section containers |
| `function_picker.rs` | Hardcoded colors, unstyled |
| `git_panel.rs` | Hardcoded colors for diff lines, unstyled buttons |
| `header_editor.rs` | Unstyled buttons, plain text_editor |
| `health.rs` | One `Message::None` stub; hardcoded status colors |
| `import_c.rs` | Hardcoded colors, unstyled |
| `makefile_editor.rs` | Unstyled buttons |
| `onboarding.rs` | Unstyled |
| `project_notes.rs` | Unstyled buttons, autosave label plain text |
| `project_search.rs` | Hardcoded match highlight colors |
| `shortcuts.rs` | Plain column, no card container |
| `stats.rs` | Hardcoded colors in metric display |
| `usage_tracker.rs` | Hardcoded colors |

---

## P3 — Feature Completeness

### Composer: No Nested Block Editing
- **Location:** `newc/src/views/main_builder.rs`
- **Status:** `IfBlock`, `WhileLoop`, and `ForLoop` blocks have a `body` Vec in their data model and appear correctly in the preview. But the UI provides no way to add, edit, or reorder child blocks inside them.
- **Fix:** When an if/while/for block is selected in the editor panel, render a nested block list with its own "Add block" controls and move up/down buttons operating on the child Vec.

### Composer: Duplicate Block Button Missing
- **Status:** README documents a "⧉ Duplicate" button per block. Not implemented in the iced port.
- **Fix:** Add `Message::ComposerBlockDuplicate(usize)` and a ⧉ button in each block row.

### Composer: Global `#include` Checkbox List Missing
- **Status:** README documents an `#include` checkbox list (auto-populated from project headers). Not present in current UI.
- **Fix:** Load project headers and show a checklist; selected includes are prepended to the generated `main.c`.

### File Watcher: No Visible Feedback
- **Location:** `newc/src/app.rs:454`
- **Status:** `Message::FileChanged(_path)` is handled but the handler is empty — no project refresh, no toast, nothing happens.
- **Fix:** On `FileChanged`, reload the affected module's source content and push an `Info` toast showing the changed file name.

### Library "From Library" in Module Detail
- **Status:** Button exists but fires `Message::None`. `function_picker.rs` view exists but is not wired to module_detail flow.
- **Fix:** See P1 `ModuleAddFromLibrary`.

### Onboarding Wizard — Project Discovery Step Unverified
- **Location:** `newc/src/views/onboarding.rs`
- **Status:** The step that scans for existing projects (`onboarding_found`) and lets the user toggle which ones to import has not been integration-tested.

### Self-Update (CLI)
- **Status:** `newc update` and `newc update --check` CLI commands exist in `cli.rs` but the updater module is a stub. No binary download logic.
- **Fix:** Port updater from egui version using `ureq` against GitHub Releases API.

### Syntax Highlighting in Header Editor
- **Status:** `header_editor.rs` uses a plain `text_editor` with no syntax highlighting.
- **Fix:** Enable iced's built-in `highlighter` feature on the `text_editor` widget using the `highlight` method with a tree-sitter or regex-based C highlighter.

### Build Panel: Clickable Diagnostics Scroll Not Verified
- **Status:** `DiagJumpTo` navigates to module_detail and sets `highlight_line`, but the line is not visually highlighted (see P0). End-to-end diagnostic click flow is broken.

---

## Known WSL2 Constraints

- GPU rendering uses LLVMpipe software Vulkan (`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`). Detected automatically at startup via `/proc/version`.
- Wayland is force-disabled (`WAYLAND_DISPLAY` unset) to avoid WSLg socket instability.
- Software rendering may cause subtle color rendering differences vs native GPU — important context for debugging the highlighting issue.
