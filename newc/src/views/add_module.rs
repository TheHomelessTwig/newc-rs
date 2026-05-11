use iced::widget::{button, column, row, text, text_input};
use iced::{Color, Element};
use newc_core::{module::is_valid_c_ident, project::Project};

use crate::state::{Message, View};

pub fn view<'a>(state: &'a crate::state::AppState, project: &'a Project) -> Element<'a, Message> {
    let name = &state.add_module_name;
    let name_valid = name.trim().is_empty() || is_valid_c_ident(name.trim());
    let can_create = !name.trim().is_empty() && name_valid;

    let mut col = column![
        row![
            button(text("← Back"))
                .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
            text(format!("+ Add Module — {}", project.name))
                .size(18)
                .color(Color::from_rgb(0.663, 0.863, 0.463)),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
        row![
            text("Module name:").width(120),
            text_input("e.g. parser", name)
                .on_input(Message::AddModuleName)
                .width(240),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(12)
    .padding(16);

    if !name_valid && !name.is_empty() {
        col = col.push(
            text("Must be a valid C identifier (letters, digits, underscores; no spaces)")
                .size(12)
                .color(Color::from_rgb(1.0, 0.3, 0.3)),
        );
    }

    let mut create_btn = button(text("Create Module"));
    if can_create {
        create_btn = create_btn.on_press(Message::AddModuleSubmit);
    }

    col.push(
        row![
            create_btn,
            button(text("Cancel"))
                .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        ]
        .spacing(8),
    )
    .into()
}
