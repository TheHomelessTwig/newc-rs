//! Keyboard shortcuts reference panel — displayed as a floating overlay when `?` is pressed.

use iced::Element;
use iced::widget::{Space, button, column, container, row, text};

use crate::state::{AppState, Message};
use crate::theme as th;

/// Renders the keyboard shortcuts overlay; returns an empty widget when `state.show_shortcuts` is false.
pub fn view(state: &AppState) -> Element<'_, Message> {
    if !state.show_shortcuts {
        return Space::new().into();
    }

    let rows: &[(&str, &str)] = &[
        ("Ctrl+P", "Quick search (projects / functions)"),
        ("Ctrl+Z", "Undo (Composer)"),
        ("Ctrl+Y / Ctrl+Shift+Z", "Redo (Composer)"),
        ("Ctrl+S", "Save (notes / module editor)"),
        ("?", "Open this shortcuts panel"),
        ("Esc", "Close modal / cancel"),
        ("↑ ↓", "Navigate quick-search list"),
        ("Enter", "Confirm quick-search selection"),
    ];

    let table: Vec<Element<Message>> = rows
        .iter()
        .map(|(key, desc)| {
            row![
                text(*key).color(th::color::yellow()).width(240),
                text(*desc).color(th::color::text()),
            ]
            .spacing(12)
            .into()
        })
        .collect();

    let inner = column![
        text("Keyboard Shortcuts").size(18).color(th::color::text()),
        column(table).spacing(6),
        button(text("Close"))
            .on_press(Message::ShowShortcuts(false))
            .style(th::btn_ghost),
    ]
    .spacing(12)
    .padding(20)
    .max_width(500);

    container(inner).style(th::card_raised_style).into()
}
