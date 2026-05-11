use iced::widget::{button, column, row, text, text_editor, Space};
use iced::{Color, Element, Length};
use newc_core::project::Project;

use crate::state::{AppState, Message, View};

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Project"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Notes — {}", project.name)).size(18),
        Space::new().width(Length::Fill),
        text("autosave").size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let editor = text_editor(&state.notes_content)
        .on_action(Message::NotesEdit)
        .height(Length::Fill);

    column![header, editor]
        .spacing(8)
        .padding(12)
        .into()
}
