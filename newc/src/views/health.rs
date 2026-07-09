//! Project health dashboard — summary cards and detail lists for build status, dead code, lint, and more.

use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Element, Length};
use newc_core::{analysis, build_history, grep, lint, project::Project};

use crate::theme as th;
use crate::state::{AppState, Message, View};

/// Cached health check results for a project, recomputed on demand.
#[derive(Default, Clone)]
pub struct HealthSnapshot {
    pub last_build_ok: bool,
    pub last_build_text: String,
    pub dead_code_count: usize,
    pub dead_code_text: String,
    pub dead_code_funcs: Vec<String>,
    pub missing_includes_count: usize,
    pub missing_includes_text: String,
    pub todos_count: usize,
    pub todos_text: String,
    /// Each entry is `(file, line_number, text)`.
    pub todos: Vec<(String, usize, String)>,
    pub lint_count: usize,
    pub lint_text: String,
    /// Each entry is `(file, lint_code, message)`.
    pub lint_warnings: Vec<(String, &'static str, String)>,
    pub header_guard_count: usize,
    pub header_guard_text: String,
    pub header_guard_files: Vec<String>,
    pub proto_mismatch_count: usize,
    pub proto_mismatch_text: String,
    pub proto_mismatches: Vec<(String, String)>,
    /// Unix timestamp of the most recently modified source file, used to detect staleness.
    pub source_mtime: u64,
}

/// Renders the health dashboard screen with summary cards and expandable detail sections.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let snap = &state.health_snapshot;

    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("♥ Health — {}", project.name))
            .size(18)
            .color(th::color::accent()),
        Space::new().width(Length::Fill),
        button(text("Refresh")).on_press(Message::RefreshProject).style(th::btn_secondary),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Summary cards
    let cards = row![
        health_card("Last Build", &snap.last_build_text, snap.last_build_ok),
        health_card("Dead Code", &snap.dead_code_text, snap.dead_code_count == 0),
        health_card("Missing Includes", &snap.missing_includes_text, snap.missing_includes_count == 0),
        health_card("TODO/FIXME", &snap.todos_text, snap.todos_count == 0),
        health_card("Lint", &snap.lint_text, snap.lint_count == 0),
        health_card("Header Guards", &snap.header_guard_text, snap.header_guard_count == 0),
        health_card("Proto Mismatches", &snap.proto_mismatch_text, snap.proto_mismatch_count == 0),
    ]
    .spacing(6)
    .wrap();

    // Detail sections — collect owned data
    let mut detail_sections: Vec<Element<Message>> = vec![];

    if !snap.todos.is_empty() {
        let todo_rows: Vec<Element<Message>> = snap.todos.iter().map(|(file, ln, txt)| {
            row![
                text(format!("{file}:{ln}")).size(11).font(iced::Font::MONOSPACE)
                    .color(th::color::text_dim()).width(180),
                text(txt.clone()).size(11).color(th::color::yellow()),
            ]
            .spacing(8)
            .into()
        }).collect();
        detail_sections.push(text("◈ TODOs & FIXMEs").size(14).color(th::color::yellow()).into());
        detail_sections.push(column(todo_rows).spacing(2).into());
    }

    if !snap.dead_code_funcs.is_empty() {
        let rows: Vec<Element<Message>> = snap.dead_code_funcs.iter().map(|f| {
            text(format!("  {f}")).size(12).font(iced::Font::MONOSPACE)
                .color(th::color::orange()).into()
        }).collect();
        detail_sections.push(text("⚠ Unreachable Functions").size(14).color(th::color::accent()).into());
        detail_sections.push(column(rows).spacing(2).into());
    }

    if !snap.lint_warnings.is_empty() {
        let rows: Vec<Element<Message>> = snap.lint_warnings.iter().map(|(file, code, msg)| {
            row![
                text(format!("[{code}]")).size(11).width(60).color(th::color::text_dim()),
                text(format!("{file}: {msg}")).size(11).color(th::color::orange()),
            ]
            .spacing(8)
            .into()
        }).collect();
        detail_sections.push(text("⚠ Lint Warnings").size(14).color(th::color::yellow()).into());
        detail_sections.push(column(rows).spacing(2).into());
    }

    if !snap.proto_mismatches.is_empty() {
        let rows: Vec<Element<Message>> = snap.proto_mismatches.iter().map(|(func, detail)| {
            row![
                text(func.clone()).size(12).font(iced::Font::MONOSPACE).width(160),
                text(detail.clone()).size(11).color(th::color::accent()),
            ]
            .spacing(8)
            .into()
        }).collect();
        detail_sections.push(text("⚠ Prototype Mismatches").size(14).color(th::color::accent()).into());
        detail_sections.push(column(rows).spacing(2).into());
    }

    if detail_sections.is_empty() {
        detail_sections.push(
            text("✓ All checks passed!").color(th::color::green()).into()
        );
    }

    column![
        header,
        cards,
        scrollable(column(detail_sections).spacing(12)).height(Length::Fill),
    ]
    .spacing(10)
    .padding(12)
    .into()
}

fn health_card<'a>(title: &'a str, value: &'a str, ok: bool) -> Element<'a, Message> {
    use iced::widget::container;
    let color = if ok { th::color::green() } else { th::color::accent() };
    let icon = if ok { "✓" } else { "⚠" };
    container(
        column![
            text(format!("{icon} {title}")).size(11).color(color),
            text(value).size(12).color(th::color::text()),
        ]
        .spacing(2)
        .padding(8),
    )
    .style(th::section_style)
    .into()
}

// Called by app.rs update() to compute health
pub fn compute_health(state: &mut AppState, project: &Project) {
    let mut snap = HealthSnapshot::default();

    // Last build
    let records = build_history::load(&project.root);
    if let Some(last) = records.last() {
        snap.last_build_ok = last.succeeded();
        snap.last_build_text = if last.succeeded() {
            format!("OK ({})", last.duration_str())
        } else {
            "Failed".into()
        };
    } else {
        snap.last_build_text = "No builds".into();
        snap.last_build_ok = false;
    }

    // Dead code
    let unreachable = analysis::check(&project.root).unwrap_or_default();
    snap.dead_code_count = unreachable.len();
    snap.dead_code_text = if unreachable.is_empty() {
        "None".into()
    } else {
        format!("{} function(s)", unreachable.len())
    };
    snap.dead_code_funcs = unreachable.into_iter().map(|f| f.name).collect();

    // TODOs (search for "TODO" and "FIXME")
    let mut todos: Vec<(String, usize, String)> = Vec::new();
    for q in ["TODO", "FIXME"] {
        for r in grep::search(&project.root, q) {
            todos.push((r.file, r.line_no, r.text));
        }
    }
    snap.todos_count = todos.len();
    snap.todos_text = if todos.is_empty() { "None".into() } else { format!("{}", todos.len()) };
    snap.todos = todos;

    // Lint warnings — scan all .c files
    let src_dir = project.root.join("src");
    let mut all_warnings: Vec<(String, &'static str, String)> = Vec::new();
    let mut guard_files: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) == Some("c")
                && let Ok(content) = std::fs::read_to_string(&path) {
                    let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    for w in lint::lint_file(&content) {
                        if w.code == "L010" {
                            guard_files.push(file_name.clone());
                        } else {
                            all_warnings.push((file_name.clone(), w.code, w.message));
                        }
                    }
                }
        }
    }
    snap.header_guard_count = guard_files.len();
    snap.header_guard_text = if guard_files.is_empty() { "None".into() } else { format!("{}", guard_files.len()) };
    snap.header_guard_files = guard_files;
    snap.lint_count = all_warnings.len();
    snap.lint_text = if all_warnings.is_empty() { "None".into() } else { format!("{}", all_warnings.len()) };
    snap.lint_warnings = all_warnings;

    // Proto mismatches
    snap.proto_mismatch_count = 0;
    snap.proto_mismatch_text = "N/A".into();

    // Missing includes (simple heuristic)
    snap.missing_includes_count = 0;
    snap.missing_includes_text = "N/A".into();

    snap.source_mtime = max_source_mtime(&project.root);

    state.health_snapshot = snap;
    state.health_computed = true;
}

fn max_source_mtime(root: &std::path::Path) -> u64 {
    let mut max: u64 = 0;
    for dir in [root.join("src"), root.join("include")] {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.filter_map(|e| e.ok()) {
                let ext = e.path().extension().and_then(|x| x.to_str()).unwrap_or("").to_string();
                if (ext == "c" || ext == "h")
                    && let Ok(meta) = e.metadata()
                        && let Ok(mtime) = meta.modified()
                            && let Ok(elapsed) = mtime.duration_since(std::time::UNIX_EPOCH) {
                                let secs_since_epoch = elapsed.as_secs();
                                if secs_since_epoch > max { max = secs_since_epoch; }
                            }
            }
        }
    }
    max
}
