use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Color, Element, Length};

use crate::build_runner::LineKind;
use crate::state::{AppState, Message};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let lines: Vec<Element<Message>> = state.build_lines.iter().map(|line| {
        let color = match line.kind {
            LineKind::Stdout => Color::from_rgb(0.85, 0.85, 0.85),
            LineKind::Stderr => Color::from_rgb(1.0, 0.392, 0.392),
            LineKind::Info => Color::from_rgb(0.392, 0.706, 1.0),
            LineKind::Done { exit_code: Some(0), .. } => Color::from_rgb(0.392, 0.863, 0.392),
            LineKind::Done { .. } => Color::from_rgb(1.0, 0.314, 0.314),
        };
        if let LineKind::Done { exit_code, duration_ms } = line.kind {
            let timing = format!("{:.1}s", duration_ms as f64 / 1000.0);
            let msg = match exit_code {
                Some(0) => format!("Build succeeded in {timing}."),
                Some(c) => format!("Build failed (exit {c}) in {timing}."),
                None => format!("Build terminated after {timing}."),
            };
            text(msg).color(color).size(12).into()
        } else if !line.text.is_empty() {
            text(line.text.as_str()).color(color).size(12).into()
        } else {
            Space::new().into()
        }
    }).collect();

    let log_content = if lines.is_empty() {
        column![text("No build output.").size(12).color(Color::from_rgb(0.5, 0.5, 0.5))]
    } else {
        column(lines).spacing(2)
    };

    column![
        row![
            text("Build Output").size(13),
            Space::new().width(Length::Fill),
            button(text("Clear").size(12)).on_press(Message::BuildPanelClear),
            button(text(if state.build_auto_scroll { "Auto-scroll ✓" } else { "Auto-scroll" }).size(12))
                .on_press(Message::BuildAutoScrollToggle),
            button(text("Kill").size(12)).on_press(Message::BuildKill),
            button(text("×").size(12)).on_press(Message::ToggleBuildPanel),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        scrollable(log_content).height(180),
    ]
    .spacing(4)
    .padding([4, 8])
    .into()
}
