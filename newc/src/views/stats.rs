use iced::widget::{button, column, row, text, Space};
use iced::{Color, Element};
use newc_core::{project::Project, stats};

use crate::state::{Message, View};

pub fn view<'a>(_state: &'a crate::state::AppState, project: &'a Project) -> Element<'a, Message> {
    let s = stats::compute(&project.root);

    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Stats — {}", project.name))
            .size(18)
            .color(Color::from_rgb(0.663, 0.863, 0.463)),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let summary = column![
        stat_row("Modules", s.module_stats.len().to_string()),
        stat_row("Total functions", s.total_functions.to_string()),
        stat_row("Lines of code (LOC)", s.total_loc.to_string()),
        stat_row("Total source lines", s.total_source_lines.to_string()),
    ]
    .spacing(4);

    let max_loc = s.module_stats.iter().map(|m| m.loc).max().unwrap_or(1).max(1);

    let module_rows: Vec<Element<Message>> = {
        let mut rows = vec![
            row![
                text("Module").width(160).color(Color::from_rgb(0.671, 0.616, 0.949)),
                text("fns").width(60).color(Color::from_rgb(0.671, 0.616, 0.949)),
                text("LOC").width(60).color(Color::from_rgb(0.671, 0.616, 0.949)),
                text("").width(160),
            ]
            .spacing(4)
            .into(),
        ];
        for m in &s.module_stats {
            let bar_pct = (m.loc as f32 / max_loc as f32 * 100.0) as usize;
            let bar: String = "█".repeat(bar_pct / 5).into();
            rows.push(
                row![
                    text(format!("◆ {}", m.name)).width(160).color(Color::from_rgb(0.663, 0.863, 0.463)),
                    text(m.functions.to_string()).width(60).color(Color::from_rgb(1.0, 0.847, 0.4)),
                    text(m.loc.to_string()).width(60).color(Color::from_rgb(1.0, 0.847, 0.4)),
                    text(bar).size(10).color(Color::from_rgb(0.663, 0.863, 0.463)).width(160),
                ]
                .spacing(4)
                .into(),
            );
        }
        rows
    };

    column![
        header,
        Space::new().height(8),
        text("Summary").size(14).color(Color::from_rgb(0.471, 0.863, 0.910)),
        summary,
        Space::new().height(8),
        text("Per-module breakdown").size(14).color(Color::from_rgb(0.471, 0.863, 0.910)),
        column(module_rows).spacing(4),
        Space::new().height(8),
        text("LOC = non-blank, non-comment lines in src/*.c files.")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(8)
    .padding(16)
    .into()
}

fn stat_row(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).width(220).color(Color::from_rgb(0.471, 0.863, 0.910)),
        text(value).color(Color::from_rgb(0.671, 0.616, 0.949)),
    ]
    .spacing(8)
    .into()
}
