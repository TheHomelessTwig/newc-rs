use iced::widget::{button, column, row, text, text_input, Space};
use iced::{Color, Element};

use crate::app::ALL_THEMES;
use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let cfg = &state.config_draft;
    let current_theme = &state.active_theme;

    // ── Editor / Terminal ──────────────────────────────────────────────────────
    let fields = column![
        text("Settings").size(20),
        row![
            text("Terminal:").width(140),
            text_input("e.g. kitty", &cfg.terminal)
                .on_input(Message::SettingsDraftTerminal)
                .width(280),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        row![
            text("Editor:").width(140),
            text_input("e.g. nvim", &cfg.editor)
                .on_input(Message::SettingsDraftEditor)
                .width(280),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(10);

    // ── Theme picker ──────────────────────────────────────────────────────────
    let theme_btns: Vec<Element<Message>> = ALL_THEMES.iter().map(|(key, label)| {
        let is_active = current_theme == *key;
        let color = if is_active {
            Color::from_rgb(0.663, 0.863, 0.463)
        } else {
            Color::WHITE
        };
        button(text(format!("{}{}", if is_active { "✓ " } else { "  " }, label)).size(12).color(color))
            .on_press(Message::ThemeSelect(key.to_string()))
            .width(180)
            .into()
    }).collect();

    // Lay them out in 3 columns
    let col1: Vec<Element<Message>> = theme_btns.into_iter()
        .enumerate()
        .filter(|(i, _)| i % 3 == 0)
        .map(|(_, e)| e)
        .collect();
    let col2: Vec<Element<Message>> = ALL_THEMES.iter().enumerate()
        .filter(|(i, _)| i % 3 == 1)
        .map(|(_, (key, label))| {
            let is_active = current_theme == *key;
            let color = if is_active { Color::from_rgb(0.663, 0.863, 0.463) } else { Color::WHITE };
            button(text(format!("{}{}", if is_active { "✓ " } else { "  " }, label)).size(12).color(color))
                .on_press(Message::ThemeSelect(key.to_string()))
                .width(180)
                .into()
        }).collect();
    let col3: Vec<Element<Message>> = ALL_THEMES.iter().enumerate()
        .filter(|(i, _)| i % 3 == 2)
        .map(|(_, (key, label))| {
            let is_active = current_theme == *key;
            let color = if is_active { Color::from_rgb(0.663, 0.863, 0.463) } else { Color::WHITE };
            button(text(format!("{}{}", if is_active { "✓ " } else { "  " }, label)).size(12).color(color))
                .on_press(Message::ThemeSelect(key.to_string()))
                .width(180)
                .into()
        }).collect();

    let theme_grid = row![
        column(col1).spacing(4),
        column(col2).spacing(4),
        column(col3).spacing(4),
    ]
    .spacing(8);

    // ── clang-format style ────────────────────────────────────────────────────
    let clang_styles = ["file", "LLVM", "Google", "Chromium", "GNU", "Microsoft"];
    let clang_btns: Vec<Element<Message>> = clang_styles.iter().map(|s| {
        let active = cfg.clang_format_style == *s;
        let color = if active { Color::from_rgb(0.663, 0.863, 0.463) } else { Color::WHITE };
        button(text(*s).size(12).color(color))
            .on_press(Message::SettingsDraftClangStyle(s.to_string()))
            .into()
    }).collect();

    // ── Save / Discard ────────────────────────────────────────────────────────
    column![
        fields,
        Space::new().height(8),
        text("Theme (applied immediately):").size(14).color(Color::from_rgb(0.471, 0.863, 0.910)),
        text("Note: Nightfly is the closest built-in theme to Monokai Pro.")
            .size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
        theme_grid,
        Space::new().height(8),
        text("clang-format style:").size(14).color(Color::from_rgb(0.471, 0.863, 0.910)),
        row(clang_btns).spacing(4).wrap(),
        Space::new().height(8),
        row![
            button(text("Save")).on_press(Message::SettingsSave),
            button(text("Discard")).on_press(Message::SettingsDiscard),
        ]
        .spacing(8),
        Space::new().height(8),
        text("Terminal + editor are used by the 'Open in Editor' button.")
            .size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
        text("Scan dirs: edit ~/.config/newc/config.toml directly.")
            .size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(8)
    .padding(16)
    .into()
}
