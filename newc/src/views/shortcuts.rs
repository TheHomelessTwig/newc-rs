use iced::widget::{button, column, row, text, Space};
use iced::{Color, Element};

use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    if !state.show_shortcuts {
        return Space::new().into();
    }

    let rows: &[(&str, &str)] = &[
        ("Ctrl+P",             "Quick search (projects / functions)"),
        ("Ctrl+Z",             "Undo (Composer)"),
        ("Ctrl+Y / Ctrl+Shift+Z", "Redo (Composer)"),
        ("Ctrl+S",             "Save (notes / module editor)"),
        ("?",                  "Open this shortcuts panel"),
        ("Esc",                "Close modal / cancel"),
        ("↑ ↓",               "Navigate quick-search list"),
        ("Enter",              "Confirm quick-search selection"),
    ];

    let table: Vec<Element<Message>> = rows.iter().map(|(key, desc)| {
        row![
            text(*key).color(Color::from_rgb(1.0, 0.847, 0.4)).width(240),
            text(*desc).color(Color::WHITE),
        ]
        .spacing(12)
        .into()
    }).collect();

    let modal = column![
        text("Keyboard Shortcuts").size(18),
        column(table).spacing(6),
        button(text("Close")).on_press(Message::ShowShortcuts(false)),
    ]
    .spacing(12)
    .padding(20)
    .max_width(500);

    modal.into()
}
