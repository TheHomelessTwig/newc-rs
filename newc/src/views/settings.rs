use iced::widget::{button, column, row, text, text_input};
use iced::{Color, Element};

use crate::state::{Message};

pub fn view(state: &crate::state::AppState) -> Element<'_, Message> {
    let cfg = &state.config_draft;

    let theme_row = row![
        text("Theme:").width(140),
        button(text("Dark"))
            .on_press(Message::SettingsDraftTheme("dark".to_string())),
        button(text("Light"))
            .on_press(Message::SettingsDraftTheme("light".to_string())),
        text(format!("(current: {})", cfg.theme))
            .size(12)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let clang_styles = ["file", "LLVM", "Google", "Chromium", "GNU", "Microsoft"];
    let _clang_btns: Vec<Element<Message>> = clang_styles.iter().map(|s| {
        button(text(*s))
            .on_press(Message::SettingsDraftTheme(s.to_string())) // reuses theme message for now
            .into()
    }).collect();

    let _scan_dirs_text = cfg.scan_dirs.join("\n");

    let scan_label = if cfg.scan_dirs.is_empty() {
        "(none)".to_string()
    } else {
        format!("{} directories", cfg.scan_dirs.len())
    };

    column![
        text("Settings").size(18),
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
        theme_row,
        row![
            text("Scan dirs:").width(140),
            text(scan_label).color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(8),
        text("(Edit scan directories in the config file: ~/.config/newc/config.toml)")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
        row![
            button(text("Save")).on_press(Message::SettingsSave),
            button(text("Discard")).on_press(Message::SettingsDiscard),
        ]
        .spacing(8),
        text("Terminal + editor used by 'Open in Editor' on project detail view.")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
        text("Scan dirs are searched on startup for existing newc projects (max depth 3).")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(12)
    .padding(16)
    .into()
}
