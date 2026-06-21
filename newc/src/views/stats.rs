//! Project statistics screen — module count, function count, LOC summary, and per-module bar chart.

use iced::widget::{button, column, row, text, Space};
use iced::{Element};
use newc_core::{project::Project, stats};

use crate::theme as th;
use crate::state::{Message, View};

/// Renders the project stats screen with summary totals and a per-module LOC breakdown.
pub fn view<'a>(_state: &'a crate::state::AppState, project: &'a Project) -> Element<'a, Message> {
    let project_stats = stats::compute(&project.root);

    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("Stats — {}", project.name))
            .size(18)
            .color(th::color::GREEN),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let summary = column![
        stat_row("Modules", project_stats.module_stats.len().to_string()),
        stat_row("Total functions", project_stats.total_functions.to_string()),
        stat_row("Lines of code (LOC)", project_stats.total_loc.to_string()),
        stat_row("Total source lines", project_stats.total_source_lines.to_string()),
    ]
    .spacing(4);

    let max_loc = project_stats.module_stats.iter().map(|m| m.loc).max().unwrap_or(1).max(1);

    let module_rows: Vec<Element<Message>> = {
        let mut rows = vec![
            row![
                text("Module").width(160).color(th::color::PURPLE),
                text("fns").width(60).color(th::color::PURPLE),
                text("LOC").width(60).color(th::color::PURPLE),
                text("").width(160),
            ]
            .spacing(4)
            .into(),
        ];
        for m in &project_stats.module_stats {
            let bar_pct = (m.loc as f32 / max_loc as f32 * 100.0) as usize;
            let bar: String = "█".repeat(bar_pct / 5).into();
            rows.push(
                row![
                    text(format!("◆ {}", m.name)).width(160).color(th::color::GREEN),
                    text(m.functions.to_string()).width(60).color(th::color::YELLOW),
                    text(m.loc.to_string()).width(60).color(th::color::YELLOW),
                    text(bar).size(10).color(th::color::GREEN).width(160),
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
        text("Summary").size(14).color(th::color::CYAN),
        summary,
        Space::new().height(8),
        text("Per-module breakdown").size(14).color(th::color::CYAN),
        column(module_rows).spacing(4),
        Space::new().height(8),
        text("LOC = non-blank, non-comment lines in src/*.c files.")
            .size(11)
            .color(th::color::TEXT_DIM),
    ]
    .spacing(8)
    .padding(16)
    .into()
}

fn stat_row(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).width(220).color(th::color::CYAN),
        text(value).color(th::color::PURPLE),
    ]
    .spacing(8)
    .into()
}
