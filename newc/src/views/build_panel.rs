//! Inline build output panel — target buttons, scrolling log, and build controls.

use iced::widget::{Space, button, column, pick_list, row, scrollable, text, text_input};
use iced::{Element, Length};
use std::path::Path;

use crate::build_runner::LineKind;
use crate::state::{AppState, Message};
use crate::theme as th;
use newc_core::diag;

/// Derive a module name (file stem, no extension) from a diagnostic's reported path.
fn module_name_from_diag_file(file: &str) -> String {
    Path::new(file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file.to_string())
}

/// Renders the collapsible build panel shown at the bottom of the main window.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let lines: Vec<Element<Message>> = state
        .build_lines
        .iter()
        .map(|line| {
            let color = match line.kind {
                LineKind::Stdout => th::color::text(),
                LineKind::Stderr => th::color::accent(),
                LineKind::Info => th::color::cyan(),
                LineKind::Done {
                    exit_code: Some(0), ..
                } => th::color::green(),
                LineKind::Done { .. } => th::color::accent(),
            };
            if let LineKind::Done {
                exit_code,
                duration_ms,
            } = line.kind
            {
                let timing = format!("{:.1}s", duration_ms as f64 / 1000.0);
                let status_message = match exit_code {
                    Some(0) => format!("Build succeeded in {timing}."),
                    Some(c) => format!("Build failed (exit {c}) in {timing}."),
                    None => format!("Build terminated after {timing}."),
                };
                text(status_message).color(color).size(12).into()
            } else if !line.text.is_empty() {
                // Lines that parse as a compiler diagnostic (file:line[:col]: kind: message)
                // become clickable, jumping straight to that line in the module editor.
                if matches!(line.kind, LineKind::Stderr | LineKind::Stdout)
                    && let Some(d) = diag::parse(std::slice::from_ref(&line.text))
                        .into_iter()
                        .next()
                {
                    let module = module_name_from_diag_file(&d.file);
                    return button(text(line.text.as_str()).color(color).size(12))
                        .on_press(Message::DiagJumpTo {
                            module,
                            line: d.line,
                        })
                        .style(th::btn_ghost)
                        .padding(0)
                        .into();
                }
                text(line.text.as_str()).color(color).size(12).into()
            } else {
                Space::new().into()
            }
        })
        .collect();

    let log_content = if lines.is_empty() {
        column![
            text("No build output.")
                .size(12)
                .color(th::color::text_dim())
        ]
    } else {
        column(lines).spacing(2)
    };

    let target_btn = |label: &'static str, target: &'static str| -> Element<Message> {
        let active = state.build_target_current == target;
        button(text(label).size(11))
            .on_press(Message::BuildStart(target.into()))
            .style(if active {
                th::btn_nav_active
            } else {
                th::btn_secondary
            })
            .into()
    };

    let valgrind_section: Option<Element<Message>> = if state.valgrind_errors.is_empty() {
        None
    } else {
        let summary = newc_core::valgrind::summarize(&state.valgrind_errors);
        let rows: Vec<Element<Message>> = state
            .valgrind_errors
            .iter()
            .map(|e| {
                let label = format!(
                    "[{}] {}{}",
                    e.kind,
                    e.text,
                    e.leaked_bytes
                        .map(|b| format!(" ({b} bytes)"))
                        .unwrap_or_default()
                );
                if let (Some(file), Some(line)) = (&e.file, e.line) {
                    let module = module_name_from_diag_file(file);
                    button(text(label).size(11).color(th::color::accent()))
                        .on_press(Message::DiagJumpTo { module, line })
                        .style(th::btn_ghost)
                        .padding(0)
                        .into()
                } else {
                    text(label).size(11).color(th::color::accent()).into()
                }
            })
            .collect();
        Some(
            column![
                text(format!(
                    "Valgrind: {} error(s), {} byte(s) leaked",
                    summary.error_count, summary.total_leaked_bytes
                ))
                .size(12)
                .color(th::color::yellow()),
                scrollable(column(rows).spacing(2)).height(100),
            ]
            .spacing(4)
            .into(),
        )
    };

    let mut panel = column![
        row![
            text("Build").size(13),
            target_btn("All", "all"),
            target_btn("Debug", "debug"),
            target_btn("Run", "run"),
            target_btn("Test", "test"),
            target_btn("Valgrind", "valgrind"),
            target_btn("Valgrind XML", "valgrind-xml"),
            target_btn("Analyse", "analyse"),
            target_btn("Cppcheck", "cppcheck"),
            target_btn("Coverage", "coverage"),
            target_btn("Clean", "clean"),
            Space::new().width(4),
            text_input("args", &state.build_run_args)
                .on_input(Message::BuildArgsChanged)
                .size(11)
                .width(120),
            Space::new().width(4),
            {
                let profile_names: Vec<String> = state
                    .project_config
                    .as_ref()
                    .map(|pc| pc.build_profiles.iter().map(|p| p.name.clone()).collect())
                    .unwrap_or_default();
                pick_list(profile_names, state.build_profile_active.clone(), |name| {
                    Message::BuildProfileSelect(Some(name))
                })
                .placeholder("profile")
                .text_size(11)
            },
            Space::new().width(4),
            button(text("🐛 Debug").size(11))
                .on_press(Message::LaunchDebugger)
                .style(th::btn_secondary),
            Space::new().width(Length::Fill),
            button(text("◀ Err").size(12))
                .on_press_maybe((!state.diagnostics.is_empty()).then_some(Message::DiagNavPrev))
                .style(th::btn_ghost),
            button(text("Err ▶").size(12))
                .on_press_maybe((!state.diagnostics.is_empty()).then_some(Message::DiagNavNext))
                .style(th::btn_ghost),
            button(text("Clear").size(12))
                .on_press(Message::BuildPanelClear)
                .style(th::btn_ghost),
            button(
                text(if state.build_auto_scroll {
                    "⇓✓"
                } else {
                    "⇓"
                })
                .size(12)
            )
            .on_press(Message::BuildAutoScrollToggle)
            .style(th::btn_ghost),
            button(text("Kill").size(12))
                .on_press(Message::BuildKill)
                .style(th::btn_danger),
            button(text("×").size(12))
                .on_press(Message::ToggleBuildPanel)
                .style(th::btn_ghost),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
        scrollable(log_content).height(160),
    ]
    .spacing(4)
    .padding([4, 8]);

    if let Some(section) = valgrind_section {
        panel = panel.push(section);
    }
    panel.into()
}
