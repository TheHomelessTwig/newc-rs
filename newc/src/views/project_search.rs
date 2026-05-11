use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Color, Element, Length};
use newc_core::project::Project;

use crate::state::{AppState, Message, View};

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Search — {}", project.name))
            .size(18)
            .color(Color::from_rgb(1.0, 0.847, 0.4)),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let search_row = row![
        text_input("Search across all .c and .h files…", &state.search_query)
            .on_input(Message::SearchQuery)
            .on_submit(Message::SearchSubmit)
            .width(360),
        button(text("Search")).on_press(Message::SearchSubmit),
        button(text("×").size(12)).on_press(Message::SearchQuery(String::new())),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    if state.search_query.is_empty() {
        return column![
            header,
            search_row,
            text("Enter a query and press Enter or click Search.")
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(10)
        .padding(16)
        .into();
    }

    if state.search_results.is_empty() {
        return column![
            header,
            search_row,
            text("No results.").color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(10)
        .padding(16)
        .into();
    }

    let count_label = text(format!("{} result(s)", state.search_results.len()))
        .size(12)
        .color(Color::from_rgb(0.5, 0.5, 0.5));

    let col_header = row![
        text("File").width(180).color(Color::WHITE),
        text("Line").width(60).color(Color::WHITE),
        text("Text").color(Color::WHITE),
    ]
    .spacing(8);

    let q = state.search_query.to_lowercase();
    let result_rows: Vec<Element<Message>> = state.search_results.iter().map(|result| {
        let module_name = result.module.clone();
        let project_clone = project.clone();
        let file_btn = button(
            text(result.file.as_str())
                .size(12)
                .color(Color::from_rgb(0.392, 0.706, 1.0))
                .font(iced::Font::MONOSPACE),
        )
        .on_press_maybe(module_name.as_ref().map(|m| {
            Message::Navigate(View::ModuleDetail {
                project: project_clone,
                module_name: m.clone(),
            })
        }));

        let highlighted = highlight_match(&result.text, &q);

        row![
            file_btn.width(180),
            text(result.line_no.to_string()).size(12).width(60)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
            text(highlighted).size(12).font(iced::Font::MONOSPACE),
        ]
        .spacing(8)
        .into()
    }).collect();

    column![
        header,
        search_row,
        count_label,
        col_header,
        scrollable(column(result_rows).spacing(4)).height(Length::Fill),
    ]
    .spacing(8)
    .padding(16)
    .into()
}

fn highlight_match(text_str: &str, _query: &str) -> String {
    // In iced without rich_text spans, just return the text as-is.
    // Highlighting will be added when rich_text widget is wired in.
    text_str.to_string()
}
