//! Makefile editor — full-text editor with a quick-toggle sidebar for common compiler flags.

use iced::widget::{button, column, container, row, text, text_editor, Space};
use iced::{Element, Length};
use newc_core::project::Project;

use crate::theme as th;
use crate::state::{AppState, Message, View};

const FLAGS: &[(&str, &str)] = &[
    ("-Wall",                    "All warnings"),
    ("-Wextra",                  "Extra warnings"),
    ("-Wpedantic",               "Strict standard"),
    ("-g",                       "Debug symbols"),
    ("-O0",                      "No optimise"),
    ("-O2",                      "Optimise O2"),
    ("-O3",                      "Optimise O3"),
    ("-fsanitize=address",       "ASan"),
    ("-fsanitize=undefined",     "UBSan"),
    ("-std=c99",                 "C99"),
    ("-std=c11",                 "C11"),
    ("-std=c17",                 "C17"),
];

/// Renders the Makefile editor screen with flag sidebar and raw text editor.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let dirty_marker = if state.makefile_dirty { " ●" } else { "" };
    let content_text = state.makefile_content.text();

    // Read active flags from the CFLAGS line in the current Makefile text
    let cflags_line = content_text.lines()
        .find(|l| l.trim_start().starts_with("CFLAGS") && l.contains('='))
        .unwrap_or("");

    let header = row![
        button(text("← Project"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("Makefile — {}{}", project.name, dirty_marker)).size(18),
        Space::new().width(Length::Fill),
        button(text("Reset")).on_press(Message::Navigate(View::MakefileEditor(project.clone()))).style(th::btn_ghost),
        button(text("Save")).on_press(Message::MakefileSave).style(th::btn_primary),
        button(text("▶ all")).on_press(Message::BuildStart("all".into())).style(th::btn_secondary),
        button(text("▶ run")).on_press(Message::BuildStart("run".into())).style(th::btn_secondary),
        button(text("▶ clean")).on_press(Message::BuildStart("clean".into())).style(th::btn_secondary),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Flags sidebar
    let flag_btns: Vec<Element<Message>> = FLAGS.iter().map(|(flag, label)| {
        let active = cflags_line.contains(flag);
        let f = flag.to_string();
        button(
            column![
                text(*label).size(10).color(if active { th::color::GREEN } else { th::color::TEXT_DIM }),
                text(*flag).size(9).font(iced::Font::MONOSPACE).color(if active { th::color::CYAN } else { th::color::TEXT_HINT }),
            ]
            .spacing(1)
        )
        .on_press(Message::MakefileToggleFlag(f))
        .style(if active { th::btn_nav_active } else { th::btn_nav_inactive })
        .into()
    }).collect();

    let flags_panel = container(
        column(flag_btns).spacing(2)
    )
    .style(th::section_style)
    .padding(6)
    .width(110);

    let editor = text_editor(&state.makefile_content)
        .on_action(Message::MakefileEdit)
        .height(Length::Fill)
        .font(iced::Font::MONOSPACE);

    column![
        header,
        row![
            flags_panel,
            editor,
        ]
        .spacing(8)
        .height(Length::Fill),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
