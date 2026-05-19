//! Usage tracker screen — shows which library functions are called across the project's source files.

use std::collections::HashMap;

use iced::widget::{button, column, row, scrollable, text, text_input, Space};
use iced::{Element, Length};
use newc_core::{function_lib::FunctionLibrary, project::Project};

use crate::theme as th;
use crate::state::{AppState, Message, View};

/// Renders the usage tracker screen listing each library function and the files that call it.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("Usage — {}", project.name))
            .size(18)
            .color(th::color::PURPLE),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let usage_map = compute_usage(project);
    let q = state.usage_search.to_lowercase();

    let search_row = row![
        text("Search:").width(70),
        text_input("filter…", &state.usage_search)
            .on_input(Message::UsageSearch)
            .width(220),
        if !state.usage_search.is_empty() {
            button(text("×").size(12)).on_press(Message::UsageSearch(String::new()))
        } else {
            button(text("×").size(12))
        },
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    if usage_map.is_empty() {
        return column![
            header,
            search_row,
            text("No library function usage found.")
                .color(th::color::TEXT_DIM),
        ]
        .spacing(8)
        .padding(16)
        .into();
    }

    // Collect owned data to avoid lifetime issues
    struct UsageEntry { fname: String, files: String }
    let entries: Vec<UsageEntry> = {
        let mut v: Vec<UsageEntry> = usage_map
            .into_iter()
            .filter(|(fname, _)| q.is_empty() || fname.to_lowercase().contains(&q))
            .map(|(fname, files)| UsageEntry { fname, files: files.join(", ") })
            .collect();
        v.sort_by(|a, b| a.fname.cmp(&b.fname));
        v
    };

    let rows: Vec<Element<Message>> = entries.into_iter().map(|e| {
        row![
            Space::new().width(8),
            text(e.fname).width(200).size(12).font(iced::Font::MONOSPACE).color(th::color::CYAN),
            text(e.files).size(12).color(th::color::TEXT_DIM),
        ]
        .spacing(4)
        .into()
    }).collect();

    let content = if rows.is_empty() {
        column![text("No matches.").color(th::color::TEXT_DIM)]
    } else {
        column(rows).spacing(4)
    };

    column![
        header,
        search_row,
        scrollable(content).height(Length::Fill),
    ]
    .spacing(8)
    .padding(16)
    .into()
}

fn compute_usage(project: &Project) -> HashMap<String, Vec<String>> {
    let src_dir = project.root.join("src");
    let Ok(entries) = std::fs::read_dir(&src_dir) else {
        return HashMap::new();
    };

    let lib = FunctionLibrary::load();
    let func_names: Vec<String> = lib.all().iter().map(|f| f.name.clone()).collect();
    let mut usage: HashMap<String, Vec<String>> = HashMap::new();

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("c"))
        .map(|e| e.path())
        .collect();
    paths.sort();

    for path in &paths {
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        for fname in &func_names {
            if content.contains(&format!("{fname}(")) {
                usage.entry(fname.clone()).or_default().push(file_name.clone());
            }
        }
    }

    usage
}
