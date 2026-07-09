//! Global quick-search overlay (Ctrl+P) — searches library functions and known projects.

use std::path::PathBuf;

use iced::widget::{Space, button, column, row, scrollable, text, text_input};
use iced::{Color, Element, Length};
use newc_core::function_lib::FunctionLibrary;

use crate::state::{AppState, Message};
use crate::theme as th;

/// Persistent state for the quick-search overlay.
#[derive(Default)]
pub struct QuickSearchState {
    /// Whether the overlay is currently visible.
    pub open: bool,
    pub query: String,
    /// Index of the highlighted result row for keyboard navigation.
    pub cursor: usize,
}

/// A single result entry in the quick-search overlay.
#[derive(Clone)]
pub enum QuickSearchResult {
    /// A function from the function library.
    Function {
        name: String,
        module: String,
        description: String,
    },
    /// A known project path.
    Project { name: String, path: PathBuf },
}

/// Renders the quick-search overlay; returns an empty widget when `state.quick_search.open` is false.
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    if !state.quick_search.open {
        return Space::new().into();
    }

    let qs = &state.quick_search;
    let lib = FunctionLibrary::load();
    let results = collect_results(&qs.query, &lib, &state.known_projects);

    let result_rows: Vec<Element<Message>> = results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let selected = i == qs.cursor;
            let _bg_color = if selected {
                Color::from_rgba(0.235, 0.392, 0.706, 0.314)
            } else {
                Color::TRANSPARENT
            };

            let content: Element<Message> = match result {
                QuickSearchResult::Function {
                    name,
                    module,
                    description,
                } => {
                    let _path_for_fn = state.known_projects.first().cloned().unwrap_or_default();
                    row![
                        text(name.clone()).font(iced::Font::MONOSPACE),
                        text(format!("({module})"))
                            .size(11)
                            .color(th::color::text_dim()),
                        Space::new().width(Length::Fill),
                        text(description.clone()).size(11).color(th::color::text()),
                    ]
                    .spacing(6)
                    .into()
                }
                QuickSearchResult::Project { name, path } => {
                    let _path_clone = path.clone();
                    row![
                        text("⌂ ").color(th::color::yellow()),
                        text(name.clone()),
                        text(path.display().to_string())
                            .size(11)
                            .color(th::color::text_dim()),
                    ]
                    .spacing(6)
                    .into()
                }
            };

            let select_message = match result {
                QuickSearchResult::Function { name, module, .. } => {
                    Message::QuickSearchSelectFunc {
                        name: name.clone(),
                        header: module.clone(),
                    }
                }
                QuickSearchResult::Project { path, .. } => {
                    let project_path = path.clone();
                    Message::OpenProject(project_path)
                }
            };

            button(content)
                .on_press(select_message)
                .width(Length::Fill)
                .into()
        })
        .collect();

    column![
        text_input(
            "Search functions, projects… (Ctrl+P to toggle, Esc to close)",
            &qs.query
        )
        .on_input(Message::QuickSearchQuery)
        .width(Length::Fill),
        scrollable(if results.is_empty() {
            column![text("No results").color(th::color::text_dim())]
        } else {
            column(result_rows).spacing(2)
        })
        .height(330),
        row![
            text("↑↓ navigate  Enter select  Esc close")
                .size(11)
                .color(th::color::text_dim()),
            Space::new().width(Length::Fill),
            text(format!("{} results", results.len()))
                .size(11)
                .color(th::color::text_dim()),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .padding(12)
    .max_width(520)
    .into()
}

fn collect_results(
    query: &str,
    lib: &FunctionLibrary,
    projects: &[std::path::PathBuf],
) -> Vec<QuickSearchResult> {
    let q = query.to_lowercase();
    let mut results = Vec::new();

    let funcs: Vec<_> = if q.is_empty() {
        lib.all().to_vec()
    } else {
        lib.search(&q).into_iter().cloned().collect()
    };

    for f in funcs.iter().take(20) {
        results.push(QuickSearchResult::Function {
            name: f.name.clone(),
            module: f.module.clone(),
            description: f.description.clone(),
        });
    }

    for path in projects {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if q.is_empty()
            || name.to_lowercase().contains(&q)
            || path.display().to_string().to_lowercase().contains(&q)
        {
            results.push(QuickSearchResult::Project {
                name,
                path: path.clone(),
            });
        }
    }

    results
}
