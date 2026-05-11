use std::path::{Path, PathBuf};

use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Color, Element, Length};
use newc_core::sync::{extract_function_implementations};

use crate::state::{AppState, Message, View};

#[derive(Default)]
pub struct ModuleDetailState {
    pub selected_func: Option<String>,
    pub edit_mode: bool,
    pub edit_buf: String,
    pub show_header_editor: bool,
    pub unreachable_funcs: Vec<String>,
    pub check_ran: bool,
    pub proto_mismatch: Option<String>,
    pub show_call_tree: bool,
    pub call_tree_lines: Vec<String>,
    pub highlight_line: Option<usize>,
}

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
    let back_msg = project.as_ref()
        .map(|p| Message::Navigate(View::ProjectDetail(p.clone())))
        .unwrap_or(Message::Navigate(View::Home));

    let header = row![
        button(text("← Project")).on_press(back_msg),
        text(format!("◆ {}", module_name))
            .size(18)
            .color(Color::from_rgb(0.663, 0.863, 0.463)),
        text(src_path.display().to_string())
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
        Space::new().width(Length::Fill),
        button(text("Edit Header (.h)").size(11)).on_press(Message::None),
        button(text("+ From Library").size(11)).on_press(Message::None),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Toolbar ───────────────────────────────────────────────────────────────
    let toolbar = row![
        button(text("✓ Check").size(12)).on_press(Message::ModuleRunCheck),
        button(text("⟳ Sync").size(12)).on_press(Message::ModuleSyncNow),
        button(text("clang-format").size(12)).on_press(Message::ModuleClangFormat),
        if mds.show_call_tree {
            button(text("Hide call tree").size(12)).on_press(Message::ModuleShowCallTree(false))
        } else {
            button(text("Call tree").size(12)).on_press(Message::ModuleShowCallTree(true))
        },
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // ── Dead-code banner ──────────────────────────────────────────────────────
    let dead_banner: Option<Element<Message>> = if mds.check_ran && !mds.unreachable_funcs.is_empty() {
        Some(
            text(format!("⚠ {} unreachable function(s): {}", mds.unreachable_funcs.len(),
                mds.unreachable_funcs.join(", ")))
                .size(12)
                .color(Color::from_rgb(1.0, 0.549, 0.0))
                .into()
        )
    } else {
        None
    };

    // ── Left: function list ───────────────────────────────────────────────────
    // Collect owned function names
    struct FuncEntry { name: String, is_unreachable: bool }
    let func_entries: Vec<FuncEntry> = funcs.iter().map(|f| FuncEntry {
        name: f.name.clone(),
        is_unreachable: mds.unreachable_funcs.contains(&f.name),
    }).collect();

    let fn_list: Vec<Element<Message>> = func_entries.iter().map(|fe| {
        let selected = mds.selected_func.as_deref() == Some(&fe.name);
        let color = if fe.is_unreachable {
            Color::from_rgb(1.0, 0.376, 0.533)
        } else if selected {
            Color::from_rgb(0.663, 0.863, 0.463)
        } else {
            Color::WHITE
        };
        button(text(fe.name.clone()).size(12).color(color))
            .on_press(Message::ModuleSelectFunc(Some(fe.name.clone())))
            .width(Length::Fill)
            .into()
    }).collect();

    let fn_sidebar = scrollable(column(fn_list).spacing(2)).width(180).height(Length::Fill);

    // ── Right: source / edit panel ────────────────────────────────────────────
    let right_panel: Element<Message> = if mds.edit_mode {
        // Edit mode: text editor for the selected function body
        column![
            row![
                text(mds.selected_func.as_deref().unwrap_or("")).size(14),
                Space::new().width(Length::Fill),
                button(text("Save").size(12))
                    .on_press(mds.selected_func.as_ref().map(|n| Message::ModuleSaveFunc {
                        name: n.clone(),
                        new_impl: mds.edit_buf.clone(),
                    }).unwrap_or(Message::None)),
                button(text("Cancel").size(12)).on_press(Message::ModuleEditMode(false)),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
            scrollable(
                text(mds.edit_buf.as_str()).font(iced::Font::MONOSPACE).size(13)
            ).height(Length::Fill),
        ]
        .spacing(8)
        .into()
    } else if let Some(selected) = &mds.selected_func {
        // View mode: show function source
        let func = funcs.iter().find(|f| &f.name == selected);
        if let Some(f) = func {
            let lint_warnings: Vec<String> = {
                let warns = newc_core::lint::lint_file(&f.body);
                warns.iter().map(|w| format!("[{}] L{}: {}", w.code, w.line_no, w.message)).collect()
            };

            let mut col = column![
                row![
                    text(f.name.clone()).size(15).color(Color::from_rgb(0.663, 0.863, 0.463)),
                    Space::new().width(Length::Fill),
                    button(text("✎ Edit").size(12))
                        .on_press(Message::ModuleEditMode(true)),
                    button(text("✗ Delete").size(12))
                        .on_press(Message::ModuleDeleteFunc(f.name.clone())),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
                text(f.signature.clone()).size(12).font(iced::Font::MONOSPACE)
                    .color(Color::from_rgb(0.471, 0.863, 0.910)),
            ]
            .spacing(6);

            if !lint_warnings.is_empty() {
                col = col.push(
                    column(lint_warnings.into_iter().map(|w| {
                        text(w).size(11).color(Color::from_rgb(1.0, 0.847, 0.4)).into()
                    }).collect::<Vec<_>>())
                    .spacing(2)
                );
            }

            col = col.push(
                scrollable(
                    text(f.body.clone()).font(iced::Font::MONOSPACE).size(13)
                ).height(Length::Fill)
            );

            // Call tree
            if mds.show_call_tree && !mds.call_tree_lines.is_empty() {
                col = col.push(text("Call tree:").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)));
                let tree_rows: Vec<Element<Message>> = mds.call_tree_lines.iter().map(|l| {
                    text(l.clone()).size(11).font(iced::Font::MONOSPACE).into()
                }).collect();
                col = col.push(scrollable(column(tree_rows).spacing(2)).height(120));
            }

            col.into()
        } else {
            text("Function not found.").into()
        }
    } else {
        // No selection — show full source
        scrollable(
            text(src_content.clone()).font(iced::Font::MONOSPACE).size(12)
        )
        .height(Length::Fill)
        .into()
    };

    let mut layout = column![header, toolbar];
    if let Some(banner) = dead_banner {
        layout = layout.push(banner);
    }
    layout
        .push(
            row![fn_sidebar, right_panel]
                .spacing(8)
                .height(Length::Fill),
        )
        .spacing(8)
        .padding(12)
        .into()
}
