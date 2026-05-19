//! Project notes screen — full-screen auto-saving text editor for free-form notes.

use iced::widget::{button, column, row, text, text_editor, Space};
use iced::{Element, Length};
use newc_core::project::Project;

use crate::theme as th;
use crate::state::{AppState, Message, View};

/// Renders the project notes editor screen.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Project"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("Notes — {}", project.name)).size(18),
        Space::new().width(Length::Fill),
        th::hint_text("autosave").size(11),
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
