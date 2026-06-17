//! New-project creation form — name, author, location, git init, template, and default modules.

use iced::widget::{button, checkbox, column, row, scrollable, text, text_input, Space};
use iced::Element;
use newc_core::{module::is_valid_c_ident, project_template};

use crate::theme as th;
use crate::state::{AppState, Message, View};

/// Renders the "New Project" creation form.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let name = &state.create_name;
    let name_valid = name.trim().is_empty() || is_valid_c_ident(name.trim());
    let loc_valid = state.create_location.trim().is_empty()
        || std::path::Path::new(&state.create_location).exists();
    let can_create = !name.trim().is_empty() && name_valid && loc_valid;

    // ── Template row ──────────────────────────────────────────────────────────
    let templates = project_template::all_templates();
    let _none_selected = state.selected_template.is_none();
    let mut tmpl_btns: Vec<Element<Message>> = vec![
        text("Template:").width(90).into(),
        button(text("Blank"))
            .on_press(Message::CreateTemplate(usize::MAX))
            .style(th::btn_secondary)
            .into(),
    ];
    for (i, t) in templates.iter().enumerate() {
        tmpl_btns.push(
            button(text(t.name).size(12))
                .on_press(Message::CreateTemplate(i))
                .style(th::btn_secondary)
                .into(),
        );
    }
    let template_row = row(tmpl_btns).spacing(6).align_y(iced::Alignment::Center);

    // ── Form fields ───────────────────────────────────────────────────────────
    let mut col = column![
        text("New Project").size(20),
        template_row,
        Space::new().height(4),
        row![
            text("Project name:").width(120),
            text_input("e.g. my_project", name)
                .on_input(Message::CreateName)
                .width(280),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(10)
    .padding(16);

    if !name_valid && !name.is_empty() {
        col = col.push(
            text("Must be a valid C identifier (letters, digits, underscores; no spaces)")
                .size(12)
                .color(th::color::ACCENT),
        );
    }

    col = col
        .push(
            row![
                text("Author:").width(120),
                text_input("e.g. Sam", &state.create_author)
                    .on_input(Message::CreateAuthor)
                    .width(280),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .push(
            row![
                text("Location:").width(120),
                text_input("~/projects", &state.create_location)
                    .on_input(Message::CreateLocation)
                    .width(240),
                button(text("Browse…").size(12))
                    .on_press(Message::CreateLocationBrowse)
                    .style(th::btn_secondary),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .push(
            row![
                text("Init git:").width(120),
                checkbox(state.create_git)
                    .on_toggle(Message::CreateGitToggle),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .push(
            row![
                text("Use CMake:").width(120),
                checkbox(state.create_use_cmake)
                    .on_toggle(Message::CreateUseCmakeToggle),
                text("(unchecked = Makefile)").size(11).color(th::color::TEXT_DIM),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );

    if !loc_valid {
        col = col.push(
            text("Location path does not exist.")
                .size(12)
                .color(th::color::ACCENT),
        );
    }

    // ── Default modules ───────────────────────────────────────────────────────
    col = col.push(text("Default modules:").size(13));
    let module_checks = row![
        checkbox(state.create_include_input).label("input")
            .on_toggle(|v| Message::CreateInclude("input".into(), v)),
        checkbox(state.create_include_math).label("math")
            .on_toggle(|v| Message::CreateInclude("math".into(), v)),
        checkbox(state.create_include_display).label("display")
            .on_toggle(|v| Message::CreateInclude("display".into(), v)),
        checkbox(state.create_include_array).label("array")
            .on_toggle(|v| Message::CreateInclude("array".into(), v)),
        checkbox(state.create_include_strings).label("strings")
            .on_toggle(|v| Message::CreateInclude("strings".into(), v)),
        checkbox(state.create_include_linked_list).label("linked_list")
            .on_toggle(|v| Message::CreateInclude("linked_list".into(), v)),
        checkbox(state.create_include_files).label("files")
            .on_toggle(|v| Message::CreateInclude("files".into(), v)),
        checkbox(state.create_include_test_utils).label("test_utils")
            .on_toggle(|v| Message::CreateInclude("test_utils".into(), v)),
    ]
    .spacing(12)
    .wrap();
    col = col.push(module_checks);

    // ── Buttons ───────────────────────────────────────────────────────────────
    let mut create_btn = button(text("Create")).style(th::btn_primary);
    if can_create {
        create_btn = create_btn.on_press(Message::CreateSubmit);
    }

    let content = col.push(Space::new().height(8))
       .push(
           row![
               create_btn,
               button(text("Cancel")).on_press(Message::Navigate(View::Home)).style(th::btn_ghost),
           ]
           .spacing(8),
       );
    scrollable(content).height(iced::Length::Fill).into()
}
