use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Color, Element};
use newc_core::{build_history, project::Project};

use crate::state::{Message, View};

pub fn view<'a>(_state: &'a crate::state::AppState, project: &'a Project) -> Element<'a, Message> {
    let records = build_history::load(&project.root);

    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Build History — {}", project.name))
            .size(18)
            .color(Color::from_rgb(0.663, 0.863, 0.463)),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    if records.is_empty() {
        return column![
            header,
            Space::new().height(16),
            text("No build history yet. Run a build first.")
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(8)
        .padding(16)
        .into();
    }

    let count_label = text(format!("{} build records (last 100 kept)", records.len()))
        .size(12)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    let col_header = row![
        text("Timestamp").width(180).color(Color::WHITE),
        text("Target").width(120).color(Color::WHITE),
        text("Result").width(100).color(Color::WHITE),
        text("Duration").width(80).color(Color::WHITE),
    ]
    .spacing(8);

    let rows: Vec<Element<Message>> = records.iter().rev().map(|rec| {
        let result_color = if rec.succeeded() {
            Color::from_rgb(0.392, 0.863, 0.392)
        } else {
            Color::from_rgb(1.0, 0.314, 0.314)
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
