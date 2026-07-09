//! Scrollable log of past build records for a project (last 100 kept).

use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Element};
use newc_core::{build_history, project::Project};

use crate::theme as th;
use crate::state::{Message, View};

/// Renders the build history table screen for the given project.
pub fn view<'a>(_state: &'a crate::state::AppState, project: &'a Project) -> Element<'a, Message> {
    let records = build_history::load(&project.root);

    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("Build History — {}", project.name))
            .size(18)
            .color(th::color::green()),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    if records.is_empty() {
        return column![
            header,
            Space::new().height(16),
            text("No build history yet. Run a build first.")
                .color(th::color::text_dim()),
        ]
        .spacing(8)
        .padding(16)
        .into();
    }

    let count_label = text(format!("{} build records (last 100 kept)", records.len()))
        .size(12)
        .color(th::color::text_dim());

    let col_header = row![
        text("Timestamp").width(180).color(th::color::text()),
        text("Target").width(120).color(th::color::text()),
        text("Result").width(100).color(th::color::text()),
        text("Duration").width(80).color(th::color::text()),
    ]
    .spacing(8);

    let rows: Vec<Element<Message>> = records.iter().rev().map(|rec| {
        let result_color = if rec.succeeded() {
            th::color::green()
        } else {
            th::color::accent()
        };
        let result_text = if rec.succeeded() {
            "✓ OK".to_string()
        } else {
            rec.exit_code.map(|c| format!("✗ exit {c}")).unwrap_or_else(|| "✗ killed".into())
        };
        row![
            text(rec.timestamp.clone()).size(12).width(180),
            text(rec.target.clone()).width(120),
            text(result_text).width(100).color(result_color),
            text(rec.duration_str()).width(80),
        ]
        .spacing(8)
        .into()
    }).collect();

    column![
        header,
        count_label,
        col_header,
        scrollable(column(rows).spacing(4)),
    ]
    .spacing(8)
    .padding(16)
    .into()
}
