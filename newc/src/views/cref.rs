//! C standard-library reference browser — searchable function list with signature and example.

use iced::widget::{button, column, pane_grid, row, scrollable, text, text_input};
use iced::{Color, Element, Length};
use newc_core::cref;

use crate::highlight::code_view;
use crate::state::Message;
use crate::theme as th;

/// Identifies which pane of the C Reference resizable pane-grid is being rendered.
#[derive(Clone, Copy)]
pub enum CRefPane {
    Headers,
    Functions,
    Detail,
}

/// Renders the C Reference browser with header list, function list, and detail panels.
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
    let header_panel = scrollable(column(header_btns).spacing(2))
        .height(Length::Fill)
        .width(160);

    // ── Function list ─────────────────────────────────────────────────────────
    let mut fn_btns: Vec<Element<Message>> =
        vec![text("Functions").size(13).color(Color::WHITE).into()];
    for func in &results {
        fn_btns.push(
            button(
                row![
                    text(func.name).size(12).color(Color::WHITE),
                    text(format!("<{}>", func.header))
                        .size(10)
                        .color(th::color::text_dim()),
                ]
                .spacing(4),
            )
            .on_press(Message::CRefSelectFunc(Some(func.name)))
            .style(crate::theme::btn_ghost)
            .width(Length::Fill)
            .into(),
        );
    }
    if results.is_empty() {
        fn_btns.push(
            text("No results.")
                .size(12)
                .color(th::color::text_dim())
                .into(),
        );
    }
    let fn_panel = scrollable(column(fn_btns).spacing(2))
        .height(Length::Fill)
        .width(180);

    // ── Detail panel ──────────────────────────────────────────────────────────
    let detail: Element<Message> = if let Some(name) = selected_func {
        if let Some(func) = cref::all_functions().iter().find(|f| f.name == name) {
            column![
                row![
                    text(func.name).size(16).color(th::color::orange()),
                    text(format!("<{}>", func.header)).color(th::color::cyan()),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                text("Signature").size(12).color(th::color::cyan()),
                code_view(
                    func.signature,
                    (state.config.code_font_size - 1.0).max(8.0),
                    None,
                    None
                ),
                text("Description").size(12).color(th::color::cyan()),
                text(func.description),
                text("Returns").size(12).color(th::color::cyan()),
                text(func.returns),
                text("Example").size(12).color(th::color::cyan()),
                code_view(
                    func.example,
                    (state.config.code_font_size - 1.0).max(8.0),
                    None,
                    None
                ),
            ]
            .spacing(8)
            .into()
        } else {
            text("Function not found").into()
        }
    } else {
        text("Select a function")
            .color(th::color::text_dim())
            .into()
    };

    let mut search_bar = row![
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
    if state.cref_window.is_none() {
        search_bar = search_bar.push(
            button(text("⊞").size(12))
                .on_press(Message::OpenCRefWindow)
                .style(crate::theme::btn_ghost),
        );
    }

    let headers_cell = std::cell::RefCell::new(Some(header_panel.into()));
    let fns_cell = std::cell::RefCell::new(Some(fn_panel.into()));
    let detail_cell = std::cell::RefCell::new(Some(scrollable(detail).height(Length::Fill).into()));

    let grid: Element<Message> = pane_grid::PaneGrid::new(&state.cref_panes, |_, pane, _| {
        let body: Element<Message> = match pane {
            CRefPane::Headers => headers_cell.borrow_mut().take().unwrap(),
            CRefPane::Functions => fns_cell.borrow_mut().take().unwrap(),
            CRefPane::Detail => detail_cell.borrow_mut().take().unwrap(),
        };
        pane_grid::Content::new(body)
    })
    .on_resize(8, Message::CRefPaneResized)
    .spacing(4)
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    column![search_bar, grid,].spacing(8).padding(12).into()
}
