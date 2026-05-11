use iced::widget::{button, checkbox, column, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length};
use newc_core::function_lib::FunctionTemplate;
use newc_core::sync::ExtractedFunction;

#[derive(Default, Debug, Clone)]
pub struct ImportState {
    pub extracted: Vec<ExtractedFunction>,
    pub selected: Vec<bool>,
    pub target_module: String,
    pub path_label: String,
}

pub fn view(state: &crate::state::AppState) -> Element<'_, crate::state::Message> {
    use crate::state::Message;

    let imp = &state.import_state;

    let header = row![
        text("Import from .c file").size(18),
        Space::new().width(Length::Fill),
        button(text("Cancel")).on_press(Message::ShowImport(false)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let pick_row = row![
        button(text("Browse .c file…")).on_press(Message::ImportPickFile),
        text(if imp.path_label.is_empty() { "" } else { &imp.path_label })
            .size(12)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
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
                    checkbox(checked)
                        .on_toggle(move |_v| Message::ImportToggleFunc(i)),
                    column![
                        text(func.name.as_str()).size(13).font(iced::Font::MONOSPACE),
                        text(func.signature.as_str())
                            .size(11)
                            .color(Color::from_rgb(0.5, 0.5, 0.5))
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
    let mut import_btn = button(text(format!("Import {selected_count} function(s)")));
    if can_import {
        import_btn = import_btn.on_press(Message::ImportSubmit);
    }

    column![
        header,
        pick_row,
        target_row,
        row![
            button(text("Select all").size(12)).on_press(Message::ImportToggleFunc(usize::MAX)),
            button(text("Deselect all").size(12)).on_press(Message::ImportToggleFunc(usize::MAX - 1)),
        ]
        .spacing(6),
        scrollable(column(func_rows).spacing(6)).height(360),
        row![import_btn, button(text("Cancel")).on_press(Message::ShowImport(false))].spacing(8),
    ]
    .spacing(10)
    .padding(16)
    .into()
}

#[allow(dead_code)]
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
