//! Header file editor — full-text editor for the `SYNC_IGNORE` block in a module's `.h` file.

use iced::widget::{button, column, row, text, text_editor, Space};
use iced::Length;

/// A single field within the interactive struct builder.
#[derive(Default, Clone)]
pub struct StructField {
    pub type_name: String,
    pub field_name: String,
    pub comment: String,
}

/// Interactive struct builder state used to compose a `typedef struct { … }` declaration.
#[derive(Default, Clone)]
pub struct StructBuilder {
    pub struct_name: String,
    pub fields: Vec<StructField>,
    pub typedef: bool,
}

/// Persistent state for the header editor view.
#[derive(Default, Clone)]
pub struct HeaderEditorState {
    /// Raw text content of the header's SYNC_IGNORE block.
    pub content: String,
    /// iced `text_editor` widget content (wraps `content`).
    pub te_content: text_editor::Content,
    pub insert_name: String,
    pub insert_type: String,
    pub insert_value: String,
    /// True when there are unsaved changes.
    pub dirty: bool,
    pub struct_builder: StructBuilder,
}

/// Renders the header editor screen (syntax-highlighted text editor for the `.h` file).
pub fn view<'a>(
    state: &'a crate::state::AppState,
) -> iced::Element<'a, crate::state::Message> {
    use crate::theme as th;
    use crate::state::Message;

    let ed = &state.header_editor_state;

    column![
        row![
            text("Header Editor").size(18),
            Space::new().width(Length::Fill),
            button(text("Save")).on_press(Message::HeaderSave).style(th::btn_primary),
            if ed.dirty {
                button(text("Discard")).on_press(Message::Navigate(crate::state::View::Home)).style(th::btn_danger)
            } else {
                button(text("Close")).on_press(Message::Navigate(crate::state::View::Home)).style(th::btn_ghost)
            },
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        text("Editing SYNC_IGNORE block — structs, enums, defines, etc.")
            .size(12)
            .color(th::color::TEXT_DIM),
        text_editor(&ed.te_content)
            .highlight("c", iced::highlighter::Theme::Base16Mocha)
            .on_action(Message::HeaderEditorAction)
            .font(iced::Font::MONOSPACE)
            .size(13)
            .height(Length::Fill),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
