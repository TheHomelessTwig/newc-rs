use iced::widget::{button, column, row, text, text_editor, Space};
use iced::{Element, Length};
use newc_core::project::Project;

use crate::state::{AppState, Message, View};

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let dirty_marker = if state.makefile_dirty { " ●" } else { "" };

    let header = row![
        button(text("← Project"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Makefile — {}{}", project.name, dirty_marker)).size(18),
        Space::new().width(Length::Fill),
        button(text("Reset")).on_press(Message::Navigate(View::MakefileEditor(project.clone()))),
        button(text("Save")).on_press(Message::MakefileSave),
        button(text("▶ all")).on_press(Message::BuildStart("all".into())),
        button(text("▶ run")).on_press(Message::BuildStart("run".into())),
        button(text("▶ clean")).on_press(Message::BuildStart("clean".into())),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let editor = text_editor(&state.makefile_content)
        .on_action(Message::MakefileEdit)
        .height(Length::Fill)
        .font(iced::Font::MONOSPACE);

    column![header, editor]
        .spacing(8)
        .padding(12)
        .into()
}
