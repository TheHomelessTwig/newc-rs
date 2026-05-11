use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::Element;

use crate::app::ALL_THEMES;
use crate::state::{AppState, Message};
use crate::theme as th;

fn form_row<'a>(label: &'a str, control: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    row![
        text(label).size(12).color(th::color::TEXT_DIM).width(150),
        control.into(),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    let cfg = &state.config_draft;
    let current_theme = &state.active_theme;

    // ── Editor & Terminal ──────────────────────────────────────────────────────
    let editor_section = container(
        column![
            th::section_title("Editor & Terminal"),
            form_row(
                "Terminal",
                text_input("e.g. kitty", &cfg.terminal)
                    .on_input(Message::SettingsDraftTerminal)
                    .style(th::input_style)
                    .width(280),
            ),
            form_row(
                "Editor",
                text_input("e.g. nvim", &cfg.editor)
                    .on_input(Message::SettingsDraftEditor)
                    .style(th::input_style)
                    .width(280),
            ),
            th::hint_text("Terminal + editor are used by the 'Open in Editor' button."),
        ]
        .spacing(8)
        .padding(12),
    )
    .style(th::section_style);

    // ── Theme picker ──────────────────────────────────────────────────────────
    let theme_btns: Vec<Element<Message>> = ALL_THEMES.iter().map(|(key, label)| {
        let is_active = current_theme == *key;
        let style = if is_active { th::btn_nav_active } else { th::btn_nav_inactive };
        let label_str = if is_active {
            format!("✓ {}", label)
        } else {
            format!("  {}", label)
        };
        button(text(label_str).size(12))
            .on_press(Message::ThemeSelect(key.to_string()))
            .style(style)
            .width(180)
            .into()
    }).collect();

    let col1: Vec<Element<Message>> = theme_btns.into_iter()
        .enumerate().filter(|(i, _)| i % 3 == 0).map(|(_, e)| e).collect();
    let col2: Vec<Element<Message>> = ALL_THEMES.iter().enumerate()
        .filter(|(i, _)| i % 3 == 1)
        .map(|(_, (key, label))| {
            let is_active = current_theme == *key;
            button(text(if is_active { format!("✓ {}", label) } else { format!("  {}", label) }).size(12))
                .on_press(Message::ThemeSelect(key.to_string()))
                .style(if is_active { th::btn_nav_active } else { th::btn_nav_inactive })
                .width(180)
                .into()
        }).collect();
    let col3: Vec<Element<Message>> = ALL_THEMES.iter().enumerate()
        .filter(|(i, _)| i % 3 == 2)
        .map(|(_, (key, label))| {
            let is_active = current_theme == *key;
            button(text(if is_active { format!("✓ {}", label) } else { format!("  {}", label) }).size(12))
                .on_press(Message::ThemeSelect(key.to_string()))
                .style(if is_active { th::btn_nav_active } else { th::btn_nav_inactive })
                .width(180)
                .into()
        }).collect();

    let appearance_section = container(
        column![
            th::section_title("Appearance"),
            th::hint_text("Nightfly is the closest built-in theme to Monokai Pro."),
            row![
                column(col1).spacing(4),
                column(col2).spacing(4),
                column(col3).spacing(4),
            ]
            .spacing(8),
        ]
        .spacing(8)
        .padding(12),
    )
    .style(th::section_style);

    // ── clang-format style ────────────────────────────────────────────────────
    let clang_styles = ["file", "LLVM", "Google", "Chromium", "GNU", "Microsoft"];
    let clang_btns: Vec<Element<Message>> = clang_styles.iter().map(|s| {
        let active = cfg.clang_format_style == *s;
        button(text(*s).size(12))
            .on_press(Message::SettingsDraftClangStyle(s.to_string()))
            .style(if active { th::btn_nav_active } else { th::btn_nav_inactive })
            .into()
    }).collect();

    let format_section = container(
        column![
            th::section_title("Code Formatting"),
            row(clang_btns).spacing(4).wrap(),
        ]
        .spacing(8)
        .padding(12),
    )
    .style(th::section_style);

    // ── Scan paths note ───────────────────────────────────────────────────────
    let paths_section = container(
        column![
            th::section_title("Scan Paths"),
            th::hint_text("Edit ~/.config/newc/config.toml directly to add scan directories."),
        ]
        .spacing(6)
        .padding(12),
    )
    .style(th::section_style);

    // ── Save / Discard ────────────────────────────────────────────────────────
    let actions = row![
        button(text("Save").size(12))
            .on_press(Message::SettingsSave)
            .style(th::btn_primary),
        button(text("Discard").size(12))
            .on_press(Message::SettingsDiscard)
            .style(th::btn_secondary),
    ]
    .spacing(8);

    column![
        row![th::heading("Settings"), Space::new().width(iced::Length::Fill)].spacing(8),
        th::separator(),
        editor_section,
        appearance_section,
        format_section,
        paths_section,
        actions,
    ]
    .spacing(10)
    .padding(16)
    .into()
}
