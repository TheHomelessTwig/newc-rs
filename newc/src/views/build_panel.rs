//! Inline build output panel — target buttons, scrolling log, and build controls.

use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::build_runner::LineKind;
use crate::theme as th;
use crate::state::{AppState, Message};

/// Renders the collapsible build panel shown at the bottom of the main window.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let lines: Vec<Element<Message>> = state.build_lines.iter().map(|line| {
        let color = match line.kind {
            LineKind::Stdout => th::color::TEXT,
            LineKind::Stderr => th::color::ACCENT,
            LineKind::Info => th::color::CYAN,
            LineKind::Done { exit_code: Some(0), .. } => th::color::GREEN,
            LineKind::Done { .. } => th::color::ACCENT,
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
        column![text("No build output.").size(12).color(th::color::TEXT_DIM)]
    } else {
        column(lines).spacing(2)
    };

    let target_btn = |label: &'static str, target: &'static str| -> Element<Message> {
        let active = state.build_target_current == target;
        button(text(label).size(11))
            .on_press(Message::BuildStart(target.into()))
            .style(if active { th::btn_nav_active } else { th::btn_secondary })
            .into()
    };

    column![
        row![
            text("Build").size(13),
            target_btn("All", "all"),
            target_btn("Debug", "debug"),
            target_btn("Run", "run"),
            target_btn("Test", "test"),
            target_btn("Valgrind", "valgrind"),
            target_btn("Analyse", "analyse"),
            target_btn("Clean", "clean"),
            Space::new().width(4),
            button(text("🐛 Debug").size(11))
                .on_press(Message::LaunchDebugger)
                .style(th::btn_secondary),
            Space::new().width(Length::Fill),
            button(text("Clear").size(12)).on_press(Message::BuildPanelClear).style(th::btn_ghost),
            button(text(if state.build_auto_scroll { "⇓✓" } else { "⇓" }).size(12))
                .on_press(Message::BuildAutoScrollToggle).style(th::btn_ghost),
            button(text("Kill").size(12)).on_press(Message::BuildKill).style(th::btn_danger),
            button(text("×").size(12)).on_press(Message::ToggleBuildPanel).style(th::btn_ghost),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
        scrollable(log_content).height(160),
    ]
    .spacing(4)
    .padding([4, 8])
    .into()
}
