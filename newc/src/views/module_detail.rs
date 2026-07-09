//! Module detail view — function list sidebar, syntax-highlighted source viewer, and inline editor.

use std::path::{Path, PathBuf};

use crate::highlight::code_view;
use crate::theme as th;
use iced::widget::{
    Space, button, column, container, pane_grid, pick_list, row, scrollable, text, text_editor,
};
use iced::{Element, Length};
use newc_core::sync::extract_function_implementations;

use crate::state::{AppState, Message, View};

/// Identifies which pane of the module detail resizable pane-grid is being rendered.
#[derive(Clone, Copy)]
pub enum ModulePane {
    Sidebar,
    Panel,
}

/// Persistent UI state for the module detail screen.
#[derive(Default)]
pub struct ModuleDetailState {
    /// Name of the currently selected function, if any.
    pub selected_func: Option<String>,
    /// True when the inline text editor is open for the selected function.
    pub edit_mode: bool,
    /// iced `text_editor` content for the function body being edited.
    pub edit_content: text_editor::Content,
    pub show_header_editor: bool,
    /// List of function names identified as unreachable after the last `Check` run.
    pub unreachable_funcs: Vec<String>,
    /// True once a dead-code check has been executed for this module.
    pub check_ran: bool,
    /// Prototype mismatch description for the selected function, if detected.
    pub proto_mismatch: Option<String>,
    /// True when the call-tree panel is expanded.
    pub show_call_tree: bool,
    /// Text lines of the call tree for the selected function.
    pub call_tree_lines: Vec<String>,
    /// Source line to scroll into view and highlight (e.g. from a diagnostic jump).
    pub highlight_line: Option<usize>,
    /// Function name awaiting delete confirmation; `None` when no confirmation is pending.
    pub delete_func_confirm: Option<String>,
    /// Most recent `clangd` hover result, shown below the editor in edit mode.
    pub hover_text: Option<String>,
}

/// Renders the module detail screen for `module_name`, showing its functions and source.
pub fn view<'a>(
    state: &'a AppState,
    module_name: &str,
    src_path: &PathBuf,
    project_root: &Path,
) -> Element<'a, Message> {
    let src_content = if src_path.exists() {
        std::fs::read_to_string(src_path).unwrap_or_default()
    } else {
        String::new()
    };

    let funcs = extract_function_implementations(&src_content);
    let mds = &state.module_detail_state;

    // ── Header ────────────────────────────────────────────────────────────────
    // Build the back view from project_root
    let project = newc_core::project::Project::open(project_root.to_path_buf()).ok();
    let back_msg = project
        .as_ref()
        .map(|p| Message::Navigate(View::ProjectDetail(p.clone())))
        .unwrap_or(Message::Navigate(View::Home));

    let header = row![
        button(text("← Project").size(12))
            .on_press(back_msg)
            .style(th::btn_ghost),
        text(format!("◆ {}", module_name))
            .size(18)
            .color(th::color::green()),
        text(src_path.display().to_string())
            .size(11)
            .color(th::color::text_hint()),
        Space::new().width(Length::Fill),
        button(text("Edit Header (.h)").size(11))
            .on_press(
                project
                    .as_ref()
                    .map(|p| Message::Navigate(View::HeaderEditor {
                        project: p.clone(),
                        module_name: module_name.to_string(),
                    }))
                    .unwrap_or(Message::None)
            )
            .style(th::btn_secondary),
        button(text("+ From Library").size(11))
            .on_press(Message::ModuleAddFromLibrary)
            .style(th::btn_secondary),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Toolbar ───────────────────────────────────────────────────────────────
    let toolbar = row![
        button(text("✓ Check").size(12))
            .on_press(Message::ModuleRunCheck)
            .style(th::btn_secondary),
        button(text("⟳ Sync").size(12))
            .on_press(Message::ModuleSyncNow)
            .style(th::btn_secondary),
        button(text("clang-format").size(12))
            .on_press(Message::ModuleClangFormat)
            .style(th::btn_secondary),
        if mds.show_call_tree {
            button(text("Hide call tree").size(12))
                .on_press(Message::ModuleShowCallTree(false))
                .style(th::btn_nav_active)
        } else {
            button(text("Call tree").size(12))
                .on_press(Message::ModuleShowCallTree(true))
                .style(th::btn_secondary)
        },
        {
            let mut other_modules: Vec<String> = std::fs::read_dir(project_root.join("src"))
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("c"))
                        .filter_map(|e| {
                            e.path()
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                        })
                        .filter(|n| n != module_name)
                        .collect()
                })
                .unwrap_or_default();
            other_modules.sort();
            pick_list(other_modules, state.split_compare_module.clone(), |name| {
                Message::SplitCompareSelect(Some(name))
            })
            .placeholder("split with…")
            .text_size(12)
        },
        button(text("×").size(12))
            .on_press(Message::SplitCompareSelect(None))
            .style(th::btn_ghost),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // ── Dead-code banner ──────────────────────────────────────────────────────
    let dead_banner: Option<Element<Message>> =
        if mds.check_ran && !mds.unreachable_funcs.is_empty() {
            Some(
                text(format!(
                    "⚠ {} unreachable function(s): {}",
                    mds.unreachable_funcs.len(),
                    mds.unreachable_funcs.join(", ")
                ))
                .size(12)
                .color(th::color::orange())
                .into(),
            )
        } else {
            None
        };

    // ── Left: function list ───────────────────────────────────────────────────
    // Collect owned function names
    struct FuncEntry {
        name: String,
        is_unreachable: bool,
    }
    let func_entries: Vec<FuncEntry> = funcs
        .iter()
        .map(|f| FuncEntry {
            name: f.name.clone(),
            is_unreachable: mds.unreachable_funcs.contains(&f.name),
        })
        .collect();

    let fn_list: Vec<Element<Message>> = func_entries
        .iter()
        .map(|fe| {
            let selected = mds.selected_func.as_deref() == Some(&fe.name);
            let color = if fe.is_unreachable {
                th::color::accent()
            } else if selected {
                th::color::green()
            } else {
                th::color::text()
            };
            let style = if selected {
                th::btn_nav_active
            } else {
                th::btn_nav_inactive
            };
            button(text(fe.name.clone()).size(12).color(color))
                .on_press(Message::ModuleSelectFunc(Some(fe.name.clone())))
                .style(style)
                .width(Length::Fill)
                .into()
        })
        .collect();

    let fn_sidebar = scrollable(column(fn_list).spacing(2))
        .width(180)
        .height(Length::Fill);

    // ── Right: source / edit panel ────────────────────────────────────────────
    let right_panel: Element<Message> = if mds.edit_mode {
        // Edit mode: interactive text editor for the selected function body
        column![
            row![
                text(mds.selected_func.as_deref().unwrap_or("")).size(14),
                Space::new().width(Length::Fill),
                button(text("Save").size(12))
                    .on_press(
                        mds.selected_func
                            .as_ref()
                            .map(|n| Message::ModuleSaveFunc {
                                name: n.clone(),
                                new_impl: mds.edit_content.text(),
                            })
                            .unwrap_or(Message::None)
                    )
                    .style(th::btn_primary),
                button(text("Cancel").size(12))
                    .on_press(Message::ModuleEditMode(false))
                    .style(th::btn_ghost),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
            row![
                button(text("ℹ Hover").size(11))
                    .on_press(Message::ModuleHoverRequest(module_name.to_string()))
                    .style(th::btn_secondary),
            ],
            text_editor(&mds.edit_content)
                .highlight("c", iced::highlighter::Theme::Base16Mocha)
                .on_action(Message::ModuleEditAction)
                .font(iced::Font::MONOSPACE)
                .size(state.config.code_font_size)
                .height(Length::Fill),
            {
                let hover_el: Element<Message> = if let Some(hover) = &mds.hover_text {
                    container(text(hover.clone()).size(11).color(th::color::cyan()))
                        .padding(6)
                        .into()
                } else {
                    Space::new().into()
                };
                hover_el
            },
        ]
        .spacing(8)
        .into()
    } else if let Some(selected) = &mds.selected_func {
        // View mode: show function source
        let func = funcs.iter().find(|f| &f.name == selected);
        if let Some(f) = func {
            let lint_warnings = newc_core::lint::lint_file(&f.body);

            let coverage_label: Option<String> =
                newc_core::coverage::module_summary(project_root, module_name)
                    .filter(|s| s.total > 0)
                    .map(|s| {
                        format!(
                            "Coverage: {:.0}% ({}/{} lines)",
                            s.percent(),
                            s.covered,
                            s.total
                        )
                    });

            let mut col = column![
                row![
                    text(f.name.clone()).size(15).color(th::color::green()),
                    Space::new().width(Length::Fill),
                    button(text("✎ Edit").size(12)).on_press(Message::ModuleEditMode(true)),
                    button(text("✏ Rename").size(12))
                        .on_press(Message::ModuleRenameStart(f.name.clone()))
                        .style(th::btn_secondary),
                    button(text("⇄ Move").size(12))
                        .on_press(Message::ModuleMoveStart(f.name.clone()))
                        .style(th::btn_secondary),
                    button(text("📝 Doc").size(12))
                        .on_press(Message::ModuleGenerateDoc(f.name.clone()))
                        .style(th::btn_secondary),
                    button(text("✗ Delete").size(12))
                        .on_press(Message::ModuleDeleteFunc(f.name.clone())),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
                text(f.signature.clone())
                    .size(12)
                    .font(iced::Font::MONOSPACE)
                    .color(th::color::cyan()),
            ]
            .spacing(6);

            if let Some(doc) = newc_core::doc::parse_doc_comment(&f.comment) {
                if !doc.brief.is_empty() {
                    col = col.push(text(doc.brief).size(12).color(th::color::text()));
                }
                for (name, desc) in &doc.params {
                    col = col.push(
                        row![
                            text("  @param").size(11).color(th::color::yellow()),
                            text(name.clone())
                                .size(11)
                                .font(iced::Font::MONOSPACE)
                                .color(th::color::green()),
                            text(desc.clone()).size(11).color(th::color::text_dim()),
                        ]
                        .spacing(6),
                    );
                }
                if let Some(ret) = &doc.returns {
                    col = col.push(
                        row![
                            text("  @return").size(11).color(th::color::yellow()),
                            text(ret.clone()).size(11).color(th::color::text_dim()),
                        ]
                        .spacing(6),
                    );
                }
            }

            if let Some(label) = coverage_label {
                col = col.push(text(label).size(11).color(th::color::green()));
            }

            if !lint_warnings.is_empty() {
                let body_lines: Vec<&str> = f.body.lines().collect();
                let function_name = f.name.clone();
                col = col.push(
                    column(
                        lint_warnings
                            .iter()
                            .map(|w| {
                                let label = format!("[{}] L{}: {}", w.code, w.line_no, w.message);
                                let has_fix =
                                    body_lines.get(w.line_no.saturating_sub(1)).is_some_and(
                                        |line| newc_core::lint::quick_fix(w.code, line).is_some(),
                                    );
                                let mut r = row![text(label).size(11).color(th::color::yellow())]
                                    .spacing(6)
                                    .align_y(iced::Alignment::Center);
                                if has_fix {
                                    r = r.push(
                                        button(text("Fix").size(10))
                                            .on_press(Message::LintQuickFix {
                                                module: module_name.to_string(),
                                                function: function_name.clone(),
                                                line_no: w.line_no,
                                                code: w.code.to_string(),
                                            })
                                            .style(th::btn_secondary),
                                    );
                                }
                                r.into()
                            })
                            .collect::<Vec<Element<Message>>>(),
                    )
                    .spacing(2),
                );
            }

            col = col.push(code_view(&f.body, state.config.code_font_size, None, None));

            // Call tree
            if mds.show_call_tree && !mds.call_tree_lines.is_empty() {
                col = col.push(text("Call tree:").size(12).color(th::color::cyan()));
                let tree_rows: Vec<Element<Message>> = mds
                    .call_tree_lines
                    .iter()
                    .map(|l| text(l.clone()).size(11).font(iced::Font::MONOSPACE).into())
                    .collect();
                col = col.push(scrollable(column(tree_rows).spacing(2)).height(120));
            }

            col.into()
        } else {
            text("Function not found.").into()
        }
    } else if let Some(other_module) = &state.split_compare_module {
        // Split view — current module's source alongside another module's, side by side.
        let other_path = project_root.join("src").join(format!("{other_module}.c"));
        let other_content = std::fs::read_to_string(&other_path).unwrap_or_default();
        row![
            column![
                text(module_name.to_string())
                    .size(12)
                    .color(th::color::text_dim()),
                code_view(
                    &src_content,
                    state.config.code_font_size,
                    mds.highlight_line,
                    Some(crate::highlight::MODULE_CODE_SCROLL)
                ),
            ]
            .spacing(2)
            .width(Length::FillPortion(1)),
            column![
                text(other_module.clone())
                    .size(12)
                    .color(th::color::text_dim()),
                code_view(&other_content, state.config.code_font_size, None, None),
            ]
            .spacing(2)
            .width(Length::FillPortion(1)),
        ]
        .spacing(8)
        .into()
    } else {
        // No selection — show full source with highlighting
        code_view(
            &src_content,
            state.config.code_font_size,
            mds.highlight_line,
            Some(crate::highlight::MODULE_CODE_SCROLL),
        )
    };

    let mut layout = column![header, toolbar];
    if let Some(banner) = dead_banner {
        layout = layout.push(banner);
    }
    if let Some(pending_delete) = &mds.delete_func_confirm {
        let pending_delete_name = pending_delete.clone();
        layout = layout.push(
            container(
                row![
                    text(format!("Delete `{pending_delete_name}`?"))
                        .size(12)
                        .color(th::color::accent()),
                    Space::new().width(Length::Fill),
                    button(text("Confirm").size(11))
                        .on_press(Message::ModuleDeleteFuncConfirm)
                        .style(th::btn_danger),
                    button(text("Cancel").size(11))
                        .on_press(Message::ModuleDeleteFunc(String::new()))
                        .style(th::btn_ghost),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding(6)
            .style(th::section_style),
        );
    }
    if state.show_rename_modal {
        use iced::widget::text_input;
        layout = layout.push(
            container(
                row![
                    text("Rename to:").size(12),
                    text_input("new_name", &state.rename_func_input)
                        .on_input(Message::ModuleRenameInput)
                        .on_submit(Message::ModuleRenameSubmit)
                        .width(200),
                    button(text("Rename").size(11))
                        .on_press(Message::ModuleRenameSubmit)
                        .style(th::btn_primary),
                    button(text("Cancel").size(11))
                        .on_press(Message::ModuleRenameStart(String::new()))
                        .style(th::btn_ghost),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding(6)
            .style(th::section_style),
        );
    }
    if state.show_move_modal {
        use iced::widget::text_input;
        layout = layout.push(
            container(
                row![
                    text(format!("Move '{}' to module:", state.move_func_name)).size(12),
                    text_input("target_module", &state.move_func_target_input)
                        .on_input(Message::ModuleMoveTargetInput)
                        .on_submit(Message::ModuleMoveSubmit)
                        .width(180),
                    button(text("Move").size(11))
                        .on_press(Message::ModuleMoveSubmit)
                        .style(th::btn_primary),
                    button(text("Cancel").size(11))
                        .on_press(Message::ModuleMoveStart(String::new()))
                        .style(th::btn_ghost),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding(6)
            .style(th::section_style),
        );
    }
    let sidebar_cell = std::cell::RefCell::new(Some(fn_sidebar.into()));
    let panel_cell = std::cell::RefCell::new(Some(right_panel));

    let grid: Element<Message> = pane_grid::PaneGrid::new(&state.module_panes, |_, pane, _| {
        let body: Element<Message> = match pane {
            ModulePane::Sidebar => sidebar_cell.borrow_mut().take().unwrap(),
            ModulePane::Panel => panel_cell.borrow_mut().take().unwrap(),
        };
        pane_grid::Content::new(body)
    })
    .on_resize(8, Message::ModulePaneResized)
    .spacing(4)
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    layout.push(grid).spacing(8).padding(12).into()
}
