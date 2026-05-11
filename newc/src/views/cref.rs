use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Color, Element, Length};
use newc_core::cref;

use crate::state::{Message};

pub fn view(state: &crate::state::AppState) -> Element<'_, Message> {
    let search = &state.cref_search;
    let selected_header = &state.cref_selected_header;
    let selected_func = state.cref_selected_func;

    let results: Vec<_> = if !search.is_empty() {
        cref::search(search)
    } else {
        match selected_header {
            Some(h) => cref::by_header(h),
            None => cref::all_functions().iter().collect(),
        }
    };

    // ── Header panel ──────────────────────────────────────────────────────────
    let _all_sel = selected_header.is_none() && search.is_empty();
    let mut header_btns: Vec<Element<Message>> = vec![
        text("Headers").size(13).color(Color::WHITE).into(),
        button(text("All").size(12))
            .on_press(Message::CRefSelectHeader(None))
            .into(),
    ];
    for h in cref::headers() {
        let count = cref::by_header(h).len();
        let label = format!("<{h}> ({count})");
        header_btns.push(
            button(text(label).size(11))
                .on_press(Message::CRefSelectHeader(Some(h.to_string())))
                .into(),
        );
    }
    let header_panel = scrollable(
        column(header_btns).spacing(2)
    )
    .height(Length::Fill)
    .width(160);

    // ── Function list ─────────────────────────────────────────────────────────
    let mut fn_btns: Vec<Element<Message>> = vec![
        text("Functions").size(13).color(Color::WHITE).into(),
    ];
    for func in &results {
        fn_btns.push(
            button(
                row![
                    text(func.name).size(12),
                    text(format!("<{}>", func.header))
                        .size(10)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                ]
                .spacing(4),
            )
            .on_press(Message::CRefSelectFunc(Some(func.name)))
            .into(),
        );
    }
    if results.is_empty() {
        fn_btns.push(text("No results.").size(12).color(Color::from_rgb(0.5, 0.5, 0.5)).into());
    }
    let fn_panel = scrollable(column(fn_btns).spacing(2))
        .height(Length::Fill)
        .width(180);

    // ── Detail panel ──────────────────────────────────────────────────────────
    let detail: Element<Message> = if let Some(name) = selected_func {
        if let Some(func) = cref::all_functions().iter().find(|f| f.name == name) {
            column![
                row![
                    text(func.name).size(16).color(Color::from_rgb(1.0, 0.796, 0.42)),
                    text(format!("<{}>", func.header))
                        .color(Color::from_rgb(0.471, 0.863, 0.910)),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                text("Signature").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
                text(func.signature).size(12).font(iced::Font::MONOSPACE),
                text("Description").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
                text(func.description),
                text("Returns").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
                text(func.returns),
                text("Example").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
                text(func.example).size(12).font(iced::Font::MONOSPACE),
            ]
            .spacing(8)
            .into()
        } else {
            text("Function not found").into()
        }
    } else {
        text("Select a function")
            .color(Color::from_rgb(0.5, 0.5, 0.5))
            .into()
    };

    let search_bar = row![
        text("C Reference").size(18),
        text_input("Search…", search)
            .on_input(Message::CRefSearch)
            .width(240),
        if !search.is_empty() {
            button(text("×").size(12)).on_press(Message::CRefSearch(String::new()))
        } else {
            button(text("×").size(12))
        },
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    column![
        search_bar,
        row![
            header_panel,
            fn_panel,
            scrollable(detail).height(Length::Fill),
        ]
        .spacing(8)
        .height(Length::Fill),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
