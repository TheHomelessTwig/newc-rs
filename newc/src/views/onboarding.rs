//! First-run onboarding wizard — project discovery, author name, and theme selection steps.

use iced::widget::{button, checkbox, column, row, scrollable, text, text_input, Space};
use iced::{Element, Length};

use crate::app::ALL_THEMES;
use crate::theme as th;
use crate::state::{AppState, Message};

/// Renders the onboarding wizard at the given step index (0 = find projects, 1 = author, 2 = theme).
pub fn view<'a>(state: &'a AppState, step: usize) -> Element<'a, Message> {
    let content: Element<Message> = match step {
        0 => step0(state),
        1 => step1(state),
        2 => step2(state),
        _ => step_done(),
    };

    column![
        progress_bar(step),
        content,
    ]
    .spacing(16)
    .padding(40)
    .into()
}

fn progress_bar<'a>(step: usize) -> Element<'a, Message> {
    let steps = ["Find Projects", "Author", "Theme"];
    let btns: Vec<Element<Message>> = steps.iter().enumerate().map(|(i, label)| {
        let color = if i == step {
            th::color::GREEN
        } else if i < step {
            th::color::GREEN
        } else {
            th::color::TEXT_DIM
        };
        text(format!("{}. {}", i + 1, label)).size(13).color(color).into()
    }).collect();
    row(btns).spacing(24).into()
}

fn step0<'a>(state: &'a AppState) -> Element<'a, Message> {
    let found = &state.onboarding_found;

    let project_list: Element<Message> = if found.is_empty() {
        text("No newc projects found in ~/projects/, ~/uni/, or ~/school/.")
            .size(13)
            .color(th::color::TEXT_DIM)
            .into()
    } else {
        let rows: Vec<Element<Message>> = found.iter().enumerate().map(|(i, (path, sel))| {
            let label = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let path_str = path.display().to_string();
            row![
                checkbox(*sel).label(label).on_toggle(move |_| Message::OnboardingToggleProject(i)),
                text(path_str).size(11).color(th::color::TEXT_DIM),
            ]
            .spacing(8)
            .into()
        }).collect();
        scrollable(column(rows).spacing(6)).height(300).into()
    };

    column![
        text("Welcome to newc").size(28).color(th::color::GREEN),
        text("A C project manager with GUI. Let's set things up.").size(14),
        Space::new().height(12),
        text("Found newc projects — select which to import:").size(14),
        project_list,
        Space::new().height(16),
        row![
            Space::new().width(Length::Fill),
            button(text("Next →")).on_press(Message::OnboardingNext).style(th::btn_primary),
        ],
    ]
    .spacing(8)
    .into()
}

fn step1<'a>(state: &'a AppState) -> Element<'a, Message> {
    column![
        text("Your name").size(22).color(th::color::GREEN),
        text("Used in generated file headers and main.c author fields.").size(13),
        Space::new().height(16),
        row![
            text("Author:").width(80),
            text_input("Full Name", &state.create_author)
                .on_input(Message::CreateAuthor)
                .width(320),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        Space::new().height(24),
        row![
            button(text("← Back")).on_press(Message::OnboardingBack).style(th::btn_ghost),
            Space::new().width(Length::Fill),
            button(text("Next →")).on_press(Message::OnboardingNext).style(th::btn_primary),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .into()
}

fn step2<'a>(state: &'a AppState) -> Element<'a, Message> {
    let theme_btns: Vec<Element<Message>> = ALL_THEMES.iter().map(|(key, label)| {
        let is_active = state.active_theme == *key;
        let color = if is_active {
            th::color::GREEN
        } else {
            th::color::TEXT
        };
        button(text(*label).size(12).color(color))
            .on_press(Message::ThemeSelect(key.to_string()))
            .into()
    }).collect();

    column![
        text("Choose a theme").size(22).color(th::color::GREEN),
        text("Can be changed any time in Settings.").size(13),
        Space::new().height(12),
        scrollable(
            iced::widget::Column::with_children(theme_btns).spacing(4)
        ).height(320),
        Space::new().height(16),
        row![
            button(text("← Back")).on_press(Message::OnboardingBack).style(th::btn_ghost),
            Space::new().width(Length::Fill),
            button(text("✓ Finish")).on_press(Message::OnboardingFinish).style(th::btn_primary),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .into()
}

fn step_done<'a>() -> Element<'a, Message> {
    column![
        text("All done!").size(28).color(th::color::GREEN),
        button(text("Go to Home")).on_press(Message::OnboardingFinish),
    ]
    .spacing(16)
    .into()
}
