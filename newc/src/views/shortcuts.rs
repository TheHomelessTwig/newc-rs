use egui::{Context, Key, Modifiers, Window};

use crate::state::AppState;

pub fn show(ctx: &Context, state: &mut AppState) {
    if !state.show_shortcuts {
        return;
    }

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        state.show_shortcuts = false;
        return;
    }

    let mut open = state.show_shortcuts;
    Window::new("Keyboard Shortcuts")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .min_width(380.0)
        .show(ctx, |ui| {
            let rows: &[(&str, &str)] = &[
                ("Ctrl+P",        "Quick search (projects / functions)"),
                ("Ctrl+Z",        "Undo (Composer)"),
                ("Ctrl+Y / Ctrl+Shift+Z", "Redo (Composer)"),
                ("Ctrl+S",        "Save (notes / module editor)"),
                ("?",             "Open this shortcuts panel"),
                ("Esc",           "Close modal / cancel"),
                ("↑ ↓",           "Navigate quick-search list"),
                ("Enter",         "Confirm quick-search selection"),
            ];

            egui::Grid::new("shortcuts_grid")
                .num_columns(2)
                .striped(true)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    for (key, desc) in rows {
                        ui.label(egui::RichText::new(*key).monospace().strong());
                        ui.label(*desc);
                        ui.end_row();
                    }
                });

            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                state.show_shortcuts = false;
            }
        });

    state.show_shortcuts = open;

    // '?' key toggle (only when no text field focused)
    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Questionmark)) {
        state.show_shortcuts = !state.show_shortcuts;
    }
}
