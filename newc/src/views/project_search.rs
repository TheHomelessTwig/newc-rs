//! Project-wide grep search — queries all `.c` and `.h` files with highlighted match display.

use iced::widget::{button, column, rich_text, row, scrollable, text, text_input};
use iced::widget::text::Span as TSpan;
use iced::{Color, Element, Length};
use newc_core::project::Project;

use crate::theme as th;
use crate::state::{AppState, Message, View};

/// Renders the project search screen with query input and highlighted results list.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("Search — {}", project.name))
            .size(18)
            .color(th::color::YELLOW),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let search_row = row![
        text_input("Search across all .c and .h files…", &state.search_query)
            .on_input(Message::SearchQuery)
            .on_submit(Message::SearchSubmit)
            .width(360),
        button(text("Search")).on_press(Message::SearchSubmit).style(th::btn_primary),
        button(text("×").size(12)).on_press(Message::SearchQuery(String::new())),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let replace_row = row![
        text_input("Replace with…", &state.search_replace)
            .on_input(Message::SearchReplaceInput)
            .width(360),
        button(text("Preview")).on_press(Message::SearchReplacePreview).style(th::btn_secondary),
        button(text("Apply")).on_press(Message::SearchReplaceApply).style(th::btn_danger),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let preview_section: Option<Element<Message>> = if state.replace_preview.is_empty() {
        None
    } else {
        let rows: Vec<Element<Message>> = state.replace_preview.iter().map(|p| {
            column![
                text(format!("{}:{}", p.file, p.line_no)).size(11).color(th::color::TEXT_DIM),
                text(format!("- {}", p.before)).size(12).font(iced::Font::MONOSPACE).color(th::color::ACCENT),
                text(format!("+ {}", p.after)).size(12).font(iced::Font::MONOSPACE).color(th::color::GREEN),
            ]
            .spacing(1)
            .into()
        }).collect();
        Some(
            scrollable(column(rows).spacing(6)).height(200).into()
        )
    };

    if state.search_query.is_empty() {
        return column![
            header,
            search_row,
            text("Enter a query and press Enter or click Search.")
                .color(th::color::TEXT_DIM),
        ]
        .spacing(10)
        .padding(16)
        .into();
    }

    if state.search_results.is_empty() {
        let mut col = column![
            header,
            search_row,
            text("No results.").color(th::color::TEXT_DIM),
            replace_row,
        ]
        .spacing(10)
        .padding(16);
        if let Some(p) = preview_section {
            col = col.push(p);
        }
        return col.into();
    }

    let count_label = text(format!("{} result(s)", state.search_results.len()))
        .size(12)
        .color(th::color::TEXT_DIM);

    let col_header = row![
        text("File").width(180).color(th::color::TEXT),
        text("Line").width(60).color(th::color::TEXT),
        text("Text").color(th::color::TEXT),
    ]
    .spacing(8);

    let q = state.search_query.to_lowercase();
    let result_rows: Vec<Element<Message>> = state.search_results.iter().map(|result| {
        let line = result.line_no;
        let file_btn = button(
            text(result.file.as_str())
                .size(12)
                .color(th::color::CYAN)
                .font(iced::Font::MONOSPACE),
        )
        .on_press_maybe(result.module.as_ref().map(|m| {
            Message::DiagJumpTo { module: m.clone(), line }
        }));

        let highlighted = highlight_match(&result.text, &q);

        row![
            file_btn.width(180),
            text(result.line_no.to_string()).size(12).width(60)
                .color(th::color::TEXT_DIM),
            highlighted,
        ]
        .spacing(8)
        .into()
    }).collect();

    let mut col = column![
        header,
        search_row,
        count_label,
        col_header,
        scrollable(column(result_rows).spacing(4)).height(Length::Fill),
        replace_row,
    ]
    .spacing(8)
    .padding(16);
    if let Some(p) = preview_section {
        col = col.push(p);
    }
    col.into()
}

fn highlight_match<'a>(text_str: &str, query: &str) -> Element<'a, Message> {
    let lower = text_str.to_lowercase();
    if !query.is_empty() {
        if let Some(pos) = lower.find(query) {
            let end = (pos + query.len()).min(text_str.len());
            let before  = text_str[..pos].to_string();
            let matched = text_str[pos..end].to_string();
            let after   = text_str[end..].to_string();
            let mut spans: Vec<TSpan<'static>> = Vec::new();
            if !before.is_empty() {
                spans.push(TSpan::new(before).font(iced::Font::MONOSPACE).size(12).color(th::color::TEXT));
            }
            spans.push(
                TSpan::new(matched).font(iced::Font::MONOSPACE).size(12)
                    .color(th::color::YELLOW)
                    .background(Color::from_rgba(1.0, 0.847, 0.0, 0.18)),
            );
            if !after.is_empty() {
                spans.push(TSpan::new(after).font(iced::Font::MONOSPACE).size(12).color(th::color::TEXT));
            }
            return rich_text(spans).into();
        }
    }
    rich_text([TSpan::<()>::new(text_str.to_string()).font(iced::Font::MONOSPACE).size(12).color(th::color::TEXT)]).into()
}
