# newc TODOs

Priority: P0 = blocking/visible bug, P1 = stub handler, P2 = theme gap, P3 = feature completeness, P4 = new feature.
Last updated: 2026-05-19 (post v0.5.x iced 0.14 port + quick wins pass).

---

## ✅ Resolved (recent)

| Item | Resolution |
|------|-----------|
| Syntax highlighting not rendering | Fixed — `highlight_c` + `rich_text` spans in `code_view()` (highlight.rs) |
| Highlighting only in module_detail | Fixed — `code_view` extracted to highlight.rs; used in library, cref, snippets, main_builder |
| Module editor edit mode read-only | Fixed — `text_editor::Content` (`edit_content`) wired in module_detail |
| Shortcuts panel not an overlay | Fixed — Stack overlay in `view_main()` with semi-transparent backdrop |
| Quick search overlay | Fixed — Stack overlay in `view_main()` |
| `ModuleClangFormat` stub | Fixed — runs `clang-format -i` on module source |
| `ModuleDeleteFunc` stub | Fixed — confirmation modal + `analysis::remove_function_from_source_pub` |
| `ModuleAddFromLibrary` stub | Fixed — navigates to library, `LibraryInsertToModule` appends impl_code |
| Git Init stub | Fixed — calls `newc_core::git::init` |
| `FileChanged` empty handler | Fixed — toast + `RefreshProject` |
| Snippet copy no feedback | Fixed — `push_toast("Copied to clipboard.")` |
| Build panel no error count | Fixed — build button label shows `Xe Yw` after failed build |
| Project search no line jump | Fixed — `DiagJumpTo` on result click (navigates + highlights line) |
| Project search no match highlight | Fixed — `rich_text` spans highlight query match in yellow |
| Module file size not shown | Fixed — `src_bytes` column in project module table |
| No lint rule for strtok | Fixed — L016: strtok() not re-entrant |
| No lint rule for realloc safety | Fixed — L017: direct realloc assignment loses pointer on NULL |

---

## P0 — Bugs

None known.

---

## P1 — Stub Handlers

None known. `UpdateCheck/Install`, `MoveToWorkspace`, `GitInit`, `ModuleClangFormat`, `ModuleDeleteFunc`, `ModuleAddFromLibrary` all implemented.

---

## P2 — Theme / Style Gaps

None known. All views use `th::` constants.

---

## P3 — Feature Completeness

None known. Composer nested blocks, #include checklist, file watcher conflict warning, and text_editor syntax highlighting all implemented.

---

## P4 — New Features (from codebase review 2026-05-19)

### Tier 1 — Done
| Feature | Status |
|---------|--------|
| Graph export | ✅ SVG export from call_graph + dep_graph, button in toolbar |
| Workspace persistence | ✅ Already implemented (config.workspaces, home view filters) |
| Report generation UI | ✅ "Report" button in project manage section, opens in editor |

### Tier 2 — Done
| Feature | Status |
|---------|--------|
| Expand snippets | ✅ Added: Error Handling, Sorting, Bit Ops, Linked List, Signal Handling, strtok_r, safe string-to-int |
| Build target selector | ✅ All/Debug/Run/Test/Valgrind/Clean buttons in build panel, active target highlighted |
| Compiler flags UI | ✅ Flag sidebar in Makefile editor, toggles CFLAGS line, active flags highlighted |
| Valgrind run | ✅ `valgrind` make target added to both Makefile templates; Valgrind button in build panel |
| Regex project search | ✅ `grep.rs` uses `regex` crate; invalid regex falls back to literal match |
| Per-project settings | ✅ `ProjectConfig` in `.newc_config.toml`; project override section in Settings |

### Tier 3 — Done
| Feature | Status |
|---------|--------|
| GDB launcher | ✅ "🐛 Debug" button in build panel launches `<terminal> gdb <binary>`; terminal flag adapted per terminal type |
| Clang analysis | ✅ `analyse` make target runs clang with extra warnings; "Analyse" button in build panel |
| Function rename | ✅ "✏ Rename" button in module_detail; project-wide rename via `refactor::rename_function`; `newc-core/src/refactor.rs` |
| Function move | ✅ "⇄ Move" button in module_detail; modal with target module input; `refactor::move_function` extracts, appends, re-syncs headers |

---

## P5 — Backlog (unimplemented, prioritised)

### Quick Wins (< 1 day each)
| Feature | Notes |
|---------|-------|
| Line numbers in `code_view` | Prefix each line with its number. Column before the `rich_text` span row. `highlight.rs` change only. |
| Clickable build output lines | Raw stderr lines in `build_panel.rs` matched against `state.diagnostics`; wrap in `button` that fires `DiagJumpTo`. |
| Run with CLI args | Text input in build panel; pass as `./$(TARGET) $(ARGS)` in Makefile `run` target or `ARGS` env var. |
| Font size preference | `code_font_size: f32` in `AppConfig`; applied to all `code_view` calls and text editors. |
| Doxygen stub generator | "📝 Doc" button in module_detail function view. Inserts `/** @brief ...\n * @param ...\n * @return ... */` above the function definition in the .c file AND above the prototype in the .h file. Re-syncs header after write. |

### Medium (2–4 days each)
| Feature | Notes |
|---------|-------|
| Project-wide search & replace | Extend `project_search.rs` with a replace input. Preview list of changes before applying. Uses regex from `grep.rs`. New `refactor::replace_in_project(root, pattern, replacement)` in `refactor.rs`. |
| gcov code coverage | Add `coverage` make target (`-fprofile-arcs -ftest-coverage`, run binary, `gcov src/*.c`). Parse `.gcov` files for hit counts. Show coverage % per function in module_detail sidebar. New `newc-core/src/coverage.rs`. |
| Valgrind XML parser | Run `valgrind --xml=yes --xml-file=/tmp/vg.xml ./$(TARGET)` in `valgrind` make target or a new `valgrind-xml` target. Parse XML in app, show structured summary: leak bytes, error count, clickable file:line via `DiagJumpTo`. |
| Assignment submission packer | Modal: student name, assignment number. Generates `Name_Project_A1.zip` containing src/, include/, Makefile, notes, report. Strips build/ and binaries. Button in project manage section. |
| Git stash | `git stash` / `git stash pop` buttons in git_panel.rs. Show stash list. `newc_core::git::stash(root)` / `stash_pop(root)`. |
| Quick-fix suggestions | In module_detail lint warnings list, add "Fix" button per warning that applies a known safe replacement inline: L001 `gets(` → `fgets(`, L006 `sprintf(` → `snprintf(`, etc. Write fixed line back to source. |
| Named build profiles | `BuildProfile { name, cflags }` list in `ProjectConfig`. Dropdown in build panel to switch. Selected profile's CFLAGS injected as `make EXTRA_CFLAGS="..."`. |

### Larger Scope
| Feature | Notes |
|---------|-------|
| Function documentation view | Parse `/** @brief/@param/@return */` comment block above selected function. Render as formatted text (not raw monospace) in the detail panel — bold tags, indented params. |
| Split code view | Show two module sources side by side using `pane_grid`. Useful alongside move/rename operations. |
| Build error navigation | Prev/Next buttons cycling `state.diagnostics`, scrolling `code_view` via `scroll_to` task. Useful after a build with many errors. |
| clangd hover types | On cursor move in text_editor, send `textDocument/hover` to clangd subprocess via LSP JSON-RPC over tokio stdio. Show type/signature tooltip. `newc/src/lsp.rs`. |

---

## Known WSL2 Constraints

- GPU rendering uses LLVMpipe software Vulkan (`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`). Detected automatically at startup via `/proc/version`.
- Wayland is force-disabled (`WAYLAND_DISPLAY` unset) to avoid WSLg socket instability.
