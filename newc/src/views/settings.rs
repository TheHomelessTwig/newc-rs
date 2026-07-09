//! Settings screen — editor/terminal paths, theme picker, clang-format style, and per-project overrides.

use iced::Element;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};

use crate::app::ALL_THEMES;
use crate::state::{AppState, Message};
use crate::theme as th;

fn form_row<'a>(label: &'a str, control: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    row![
        text(label).size(12).color(th::color::text_dim()).width(150),
        control.into(),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Renders the settings screen with global and per-project configuration sections.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let draft_config = &state.config_draft;
    let current_theme = &state.active_theme;

    // ── Editor & Terminal ──────────────────────────────────────────────────────
    let editor_section = container(
        column![
            th::section_title("Editor & Terminal"),
            form_row(
                "Terminal",
                text_input("e.g. kitty", &draft_config.terminal)
                    .on_input(Message::SettingsDraftTerminal)
                    .style(th::input_style)
                    .width(280),
            ),
            form_row(
                "Editor",
                text_input("e.g. nvim", &draft_config.editor)
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
    let theme_btns: Vec<Element<Message>> = ALL_THEMES
        .iter()
        .map(|(key, label)| {
            let is_active = current_theme == *key;
            let style = if is_active {
                th::btn_nav_active
            } else {
                th::btn_nav_inactive
            };
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
        })
        .collect();

    let col1: Vec<Element<Message>> = theme_btns
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i % 3 == 0)
        .map(|(_, e)| e)
        .collect();
    let col2: Vec<Element<Message>> = ALL_THEMES
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 3 == 1)
        .map(|(_, (key, label))| {
            let is_active = current_theme == *key;
            button(
                text(if is_active {
                    format!("✓ {}", label)
                } else {
                    format!("  {}", label)
                })
                .size(12),
            )
            .on_press(Message::ThemeSelect(key.to_string()))
            .style(if is_active {
                th::btn_nav_active
            } else {
                th::btn_nav_inactive
            })
            .width(180)
            .into()
        })
        .collect();
    let col3: Vec<Element<Message>> = ALL_THEMES
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 3 == 2)
        .map(|(_, (key, label))| {
            let is_active = current_theme == *key;
            button(
                text(if is_active {
                    format!("✓ {}", label)
                } else {
                    format!("  {}", label)
                })
                .size(12),
            )
            .on_press(Message::ThemeSelect(key.to_string()))
            .style(if is_active {
                th::btn_nav_active
            } else {
                th::btn_nav_inactive
            })
            .width(180)
            .into()
        })
        .collect();

    let appearance_section = container(
        column![
            th::section_title("Appearance"),
            th::hint_text("Themes restyle the whole app, including code colours."),
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
    let clang_btns: Vec<Element<Message>> = clang_styles
        .iter()
        .map(|s| {
            let active = draft_config.clang_format_style == *s;
            button(text(*s).size(12))
                .on_press(Message::SettingsDraftClangStyle(s.to_string()))
                .style(if active {
                    th::btn_nav_active
                } else {
                    th::btn_nav_inactive
                })
                .into()
        })
        .collect();

    let format_section = container(
        column![
            th::section_title("Code Formatting"),
            row(clang_btns).spacing(4).wrap(),
        ]
        .spacing(8)
        .padding(12),
    )
    .style(th::section_style);

    // ── Code font size ────────────────────────────────────────────────────────
    let font_size_section = container(
        column![
            th::section_title("Code Font Size"),
            row![
                button(text("−").size(14))
                    .on_press(Message::SettingsDraftCodeFontSize(
                        (draft_config.code_font_size - 1.0).max(8.0)
                    ))
                    .style(th::btn_secondary),
                text(format!("{:.0}px", draft_config.code_font_size)).size(13),
                button(text("+").size(14))
                    .on_press(Message::SettingsDraftCodeFontSize(
                        (draft_config.code_font_size + 1.0).min(32.0)
                    ))
                    .style(th::btn_secondary),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
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

    // ── Updates ───────────────────────────────────────────────────────────────
    let mut update_rows = column![
        text(format!("Version: v{}", crate::updater::current_version())).size(12),
        button(text("Check for Updates").size(12))
            .on_press(Message::UpdateCheck)
            .style(th::btn_secondary),
    ]
    .spacing(8);
    if let Some(ver) = &state.update_available {
        update_rows = update_rows.push(
            row![
                text(format!("v{ver} available"))
                    .size(12)
                    .color(th::color::accent()),
                button(text("Install Update").size(12))
                    .on_press(Message::UpdateInstall(ver.clone()))
                    .style(th::btn_primary),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    }
    let updates_section = container(
        column![th::section_title("Updates"), update_rows,]
            .spacing(8)
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

    // ── Per-project overrides ─────────────────────────────────────────────────
    let project_section: Option<Element<Message>> = state.project_config.as_ref().map(|_| {
        let draft = &state.project_config_draft;
        container(
            column![
                th::section_title("Project Overrides"),
                th::hint_text(
                    "Stored in .newc_config.toml in the project root. Blank = use global."
                ),
                form_row(
                    "Editor:",
                    text_input(&state.config.editor, draft.editor.as_deref().unwrap_or(""),)
                        .on_input(Message::ProjectConfigDraftEditor)
                        .width(280)
                ),
                form_row(
                    "Terminal:",
                    text_input(
                        &state.config.terminal,
                        draft.terminal.as_deref().unwrap_or(""),
                    )
                    .on_input(Message::ProjectConfigDraftTerminal)
                    .width(280)
                ),
                form_row(
                    "clang-format style:",
                    text_input(
                        &state.config.clang_format_style,
                        draft.clang_format_style.as_deref().unwrap_or(""),
                    )
                    .on_input(Message::ProjectConfigDraftClangStyle)
                    .width(180)
                ),
                th::section_title("Build Profiles"),
                column(
                    draft
                        .build_profiles
                        .iter()
                        .map(|p| {
                            row![
                                text(p.name.clone()).size(12).width(120),
                                text(p.cflags.clone()).size(11).color(th::color::text_dim()),
                                button(text("Remove").size(10))
                                    .on_press(Message::ProfileRemove(p.name.clone()))
                                    .style(th::btn_danger),
                            ]
                            .spacing(8)
                            .align_y(iced::Alignment::Center)
                            .into()
                        })
                        .collect::<Vec<Element<Message>>>()
                )
                .spacing(4),
                row![
                    text_input("name", &state.profile_name_input)
                        .on_input(Message::ProfileNameInput)
                        .width(120),
                    text_input("extra CFLAGS", &state.profile_cflags_input)
                        .on_input(Message::ProfileCflagsInput)
                        .width(280),
                    button(text("Add Profile").size(12))
                        .on_press(Message::ProfileAdd)
                        .style(th::btn_secondary),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    button(text("Save Project Settings").size(12))
                        .on_press(Message::ProjectConfigSave)
                        .style(th::btn_primary),
                    button(text("Clear").size(12))
                        .on_press(Message::ProjectConfigClear)
                        .style(th::btn_danger),
                ]
                .spacing(8),
            ]
            .spacing(6)
            .padding(12),
        )
        .style(th::section_style)
        .into()
    });

    let mut col = column![
        row![
            th::heading("Settings"),
            Space::new().width(iced::Length::Fill)
        ]
        .spacing(8),
        th::separator(),
        editor_section,
        appearance_section,
        format_section,
        font_size_section,
        paths_section,
        updates_section,
    ]
    .spacing(10)
    .padding(16);

    if let Some(proj_sec) = project_section {
        col = col.push(proj_sec);
    }
    scrollable(col.push(actions))
        .height(iced::Length::Fill)
        .into()
}
