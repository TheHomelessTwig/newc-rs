//! Import-from-C-file dialog — pick a `.c` file, select functions, and copy them into a module.

use iced::widget::{Space, button, checkbox, column, row, scrollable, text, text_input};
use iced::{Element, Length};
use newc_core::function_lib::FunctionTemplate;
use newc_core::sync::ExtractedFunction;

/// Transient state for the "Import from .c file" dialog.
#[derive(Default, Debug, Clone)]
pub struct ImportState {
    /// Functions extracted from the selected source file.
    pub extracted: Vec<ExtractedFunction>,
    /// Parallel boolean per extracted function — true when selected for import.
    pub selected: Vec<bool>,
    /// Destination module name to receive the imported functions.
    pub target_module: String,
    /// Display name of the browsed source file path.
    pub path_label: String,
}

/// Renders the import-from-.c-file dialog screen.
pub fn view(state: &crate::state::AppState) -> Element<'_, crate::state::Message> {
    use crate::state::Message;
    use crate::theme as th;

    let imp = &state.import_state;

    let header = row![
        text("Import from .c file").size(18),
        Space::new().width(Length::Fill),
        button(text("Cancel"))
            .on_press(Message::ShowImport(false))
            .style(th::btn_ghost),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let pick_row = row![
        button(text("Browse .c file…"))
            .on_press(Message::ImportPickFile)
            .style(th::btn_secondary),
        text(if imp.path_label.is_empty() {
            ""
        } else {
            &imp.path_label
        })
        .size(12)
        .color(th::color::text_dim()),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    if imp.extracted.is_empty() {
        let hint = if !imp.path_label.is_empty() {
            "No functions found in the selected file."
        } else {
            ""
        };
        return column![header, pick_row, text(hint)]
            .spacing(10)
            .padding(16)
            .into();
    }

    let target_row = row![
        text("Target module:").width(130),
        text_input("module name", &imp.target_module)
            .on_input(Message::ImportTargetModule)
            .width(200),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let selected_count = imp.selected.iter().filter(|&&s| s).count();

    let func_rows: Vec<Element<Message>> = imp
        .extracted
        .iter()
        .enumerate()
        .map(|(i, func)| {
            let checked = imp.selected.get(i).copied().unwrap_or(false);
            column![
                row![
                    checkbox(checked).on_toggle(move |_v| Message::ImportToggleFunc(i)),
                    column![
                        text(func.name.as_str())
                            .size(13)
                            .font(iced::Font::MONOSPACE),
                        text(func.signature.as_str())
                            .size(11)
                            .color(th::color::text_dim())
                            .font(iced::Font::MONOSPACE),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .into()
        })
        .collect();

    let can_import = selected_count > 0 && !imp.target_module.trim().is_empty();
    let mut import_btn =
        button(text(format!("Import {selected_count} function(s)"))).style(th::btn_primary);
    if can_import {
        import_btn = import_btn.on_press(Message::ImportSubmit);
    }

    column![
        header,
        pick_row,
        target_row,
        row![
            button(text("Select all").size(12))
                .on_press(Message::ImportToggleFunc(usize::MAX))
                .style(th::btn_ghost),
            button(text("Deselect all").size(12))
                .on_press(Message::ImportToggleFunc(usize::MAX - 1))
                .style(th::btn_ghost),
        ]
        .spacing(6),
        scrollable(column(func_rows).spacing(6)).height(360),
        row![
            import_btn,
            button(text("Cancel"))
                .on_press(Message::ShowImport(false))
                .style(th::btn_ghost)
        ]
        .spacing(8),
    ]
    .spacing(10)
    .padding(16)
    .into()
}

pub fn build_templates(state: &ImportState) -> Vec<FunctionTemplate> {
    state
        .extracted
        .iter()
        .enumerate()
        .filter(|(i, _)| state.selected.get(*i).copied().unwrap_or(false))
        .map(|(_, f)| FunctionTemplate {
            name: f.name.clone(),
            module: state.target_module.clone(),
            description: String::new(),
            signature: f.signature.clone(),
            header_code: f.signature.clone() + ";",
            impl_code: f.body.clone(),
            requires: Vec::new(),
            tags: Vec::new(),
            notes: String::new(),
            starred: false,
        })
        .collect()
}
