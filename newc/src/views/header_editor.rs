use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Color, Length};

#[derive(Default, Clone)]
pub struct StructField {
    pub type_name: String,
    pub field_name: String,
    pub comment: String,
}

#[derive(Default, Clone)]
pub struct StructBuilder {
    pub struct_name: String,
    pub fields: Vec<StructField>,
    pub typedef: bool,
}

#[derive(Default, Clone)]
pub struct HeaderEditorState {
    pub content: String,
    pub insert_name: String,
    pub insert_type: String,
    pub insert_value: String,
    pub dirty: bool,
    pub struct_builder: StructBuilder,
}

pub fn view<'a>(
    state: &'a crate::state::AppState,
) -> iced::Element<'a, crate::state::Message> {
    use crate::state::Message;

    let ed = &state.header_editor_state;

    column![
        row![
            text("Header Editor").size(18),
            Space::new().width(Length::Fill),
            button(text("Save")).on_press(Message::HeaderSave),
            if ed.dirty {
                button(text("Discard")).on_press(Message::Navigate(crate::state::View::Home))
            } else {
                button(text("Close")).on_press(Message::Navigate(crate::state::View::Home))
            },
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        text("Editing SYNC_IGNORE block — structs, enums, defines, etc.")
            .size(12)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
        scrollable(
            text(ed.content.as_str()).font(iced::Font::MONOSPACE).size(13)
        ).height(Length::Fill),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
