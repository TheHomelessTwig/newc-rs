//! Inline function picker widget — searchable checklist of library functions with dependency resolution.

use iced::widget::{checkbox, column, row, scrollable, text, text_input, Space};
use iced::{Element};
use newc_core::function_lib::FunctionLibrary;

use crate::theme as th;
use crate::state::Message;

/// Renders the function picker panel used when inserting functions from the library into a module.
pub fn view<'a>(
    state: &'a crate::state::AppState,
    lib: &'a FunctionLibrary,
) -> Element<'a, Message> {
    let search = &state.func_search;
    let selected = &state.func_selected;

    let candidates: Vec<_> = if search.is_empty() {
        lib.all().to_vec()
    } else {
        lib.search(search).into_iter().cloned().collect()
    };

    let resolved = lib.resolve_deps(selected);

    let search_row = row![
        text("Search:").width(70),
        text_input("filter functions…", search)
            .on_input(|s| Message::LibrarySearch(s))
            .width(220),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Collect all (module, name, description, is_dep, is_checked) upfront to avoid lifetime issues
    let mut modules: Vec<String> = candidates.iter().map(|f| f.module.clone()).collect();
    modules.sort();
    modules.dedup();

    struct FuncRow {
        module: String,
        name: String,
        description: String,
        is_dep: bool,
        checked: bool,
    }

    let func_data: Vec<FuncRow> = modules.iter().flat_map(|module| {
        candidates
            .iter()
            .filter(|f| &f.module == module)
            .map(|func| FuncRow {
                module: module.clone(),
                name: func.name.clone(),
                description: func.description.clone(),
                is_dep: resolved.contains(&func.name) && !selected.contains(&func.name),
                checked: selected.contains(&func.name) || resolved.contains(&func.name),
            })
            .collect::<Vec<_>>()
    }).collect();

    // Group headers
    let mut func_rows: Vec<Element<Message>> = Vec::new();
    let mut current_module = String::new();
    for fd in func_data {
        if fd.module != current_module {
            current_module = fd.module.clone();
            func_rows.push(
                text(fd.module).size(13).color(th::color::CYAN).into(),
            );
        }
        let name = fd.name.clone();
        let name2 = fd.name.clone();
        let select_checkbox = checkbox(fd.checked).on_toggle(move |checked| {
            if checked {
                Message::LibrarySelect(Some(name.clone()))
            } else {
                Message::LibrarySelect(None)
            }
        });
        let label_color = if fd.is_dep { th::color::PURPLE } else { th::color::TEXT };
        func_rows.push(
            row![
                Space::new().width(20),
                select_checkbox,
                text(name2).color(label_color).size(13),
                text(fd.description).size(11).color(th::color::TEXT_DIM),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into(),
        );
    }

    let sel_summary = if !selected.is_empty() {
        format!("{} selected ({} with deps)", selected.len(), resolved.len())
    } else {
        String::new()
    };

    column![
        search_row,
        scrollable(column(func_rows).spacing(4)).height(260),
        text(sel_summary).size(12).color(th::color::TEXT_DIM),
    ]
    .spacing(8)
    .into()
}
