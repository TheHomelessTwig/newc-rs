//! Function library browser — grouped function list, detail/edit panel, and composer integration.

use iced::widget::{button, column, pane_grid, row, scrollable, text, text_editor, text_input, Space};
use iced::{Color, Element, Length};
use newc_core::function_lib::{FunctionLibrary, FunctionTemplate};
use newc_core::main_builder::MainBlock;

use crate::theme as th;
use crate::highlight::code_view;
use crate::state::{AppState, LibraryField, Message};

/// Identifies which pane of the Library resizable pane-grid is being rendered.
#[derive(Clone, Copy)]
pub enum LibraryPane { Groups, Functions, Detail }

/// Persistent UI state for the function library view.
#[derive(Default, Clone)]
pub struct LibraryState {
    /// Name of the currently selected function, if any.
    pub selected: Option<String>,
    pub search: String,
    pub edit_mode: bool,
    /// In-progress edits to a function template (new or existing).
    pub draft: Option<FunctionTemplate>,
    /// True when the "New Function" form is open.
    pub adding_new: bool,
    /// Currently active group filter; `None` = show all.
    pub active_group: Option<String>,
    pub rename_input: String,
    /// Parameter list parsed from the draft signature for the param builder UI.
    pub draft_params: Vec<(String, String)>,
    pub draft_return_type: String,
    /// When true the signature field is edited directly, bypassing the param builder.
    pub draft_override_sig: bool,
    pub draft_params_ready: bool,
    /// Editable text-editor content for the draft's header (`.h`) code.
    pub header_editor: text_editor::Content,
    /// Editable text-editor content for the draft's implementation (`.c`) code.
    pub impl_editor: text_editor::Content,
}

impl LibraryState {
    pub fn new_draft() -> FunctionTemplate {
        FunctionTemplate {
            name: String::new(),
            module: String::new(),
            description: String::new(),
            signature: String::new(),
            header_code: String::new(),
            impl_code: String::new(),
            requires: Vec::new(),
            tags: Vec::new(),
            notes: String::new(),
            starred: false,
        }
    }

    pub fn init_params_from_sig(&mut self, sig: &str) {
        let (ret, params) = parse_sig(sig);
        self.draft_return_type = ret;
        self.draft_params = params;
        self.draft_params_ready = true;
    }

    pub fn reset_builder(&mut self) {
        self.draft_params.clear();
        self.draft_return_type = "void".to_string();
        self.draft_override_sig = false;
        self.draft_params_ready = true;
    }
}

/// Renders the function library screen with group sidebar, function list, and detail/edit panel.
pub fn view<'a>(state: &'a AppState, lib: &'a FunctionLibrary) -> Element<'a, Message> {
    let ls = &state.library_state;

    // ── Top toolbar ───────────────────────────────────────────────────────────
    let mut toolbar = row![
        text("Function Library").size(18),
        text_input("Search…", &ls.search)
            .on_input(Message::LibrarySearch)
            .width(220),
        button(text("+ New Function").size(12))
            .on_press(Message::LibraryAddingNew(true)),
        button(text("Import .c…").size(12))
            .on_press(Message::ShowImport(true)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);
    if state.library_window.is_none() {
        toolbar = toolbar.push(
            button(text("⊞").size(12))
                .on_press(Message::OpenLibraryWindow)
                .style(crate::theme::btn_ghost),
        );
    }

    // ── Groups sidebar ────────────────────────────────────────────────────────
    let _all_sel = ls.active_group.is_none();
    let starred_count = lib.all().iter().filter(|f| f.starred).count();
    let _starred_sel = ls.active_group.as_deref() == Some("__starred__");

    let mut group_btns: Vec<Element<Message>> = vec![
        row![
            text("Groups").size(13).color(Color::WHITE),
            Space::new().width(Length::Fill),
            button(text("+").size(11)).on_press(Message::LibraryGroupNew),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .into(),
        button(text("All").size(12))
            .on_press(Message::LibraryGroupSelect(None))
            .width(Length::Fill)
            .into(),
        button(text(format!("★ Starred ({})", starred_count)).size(12))
            .on_press(Message::LibraryGroupSelect(Some("__starred__".into())))
            .width(Length::Fill)
            .into(),
    ];

    for group in &lib.groups {
        let count = lib.by_module(&group.name).len();
        let label = format!("{} ({})", group.name, count);
        let name = group.name.clone();
        group_btns.push(
            button(text(label).size(12))
                .on_press(Message::LibraryGroupSelect(Some(name)))
                .width(Length::Fill)
                .into(),
        );
    }

    let group_panel = scrollable(column(group_btns).spacing(2))
        .width(160)
        .height(Length::Fill);

    // ── Function list ─────────────────────────────────────────────────────────
    let candidates: Vec<_> = {
        let by_group: Vec<_> = match ls.active_group.as_deref() {
            Some("__starred__") => lib.all().iter().filter(|f| f.starred).collect(),
            Some(g) => lib.all().iter().filter(|f| f.module == g).collect(),
            None => lib.all().iter().collect(),
        };
        if ls.search.is_empty() {
            by_group
        } else {
            let q = ls.search.to_lowercase();
            by_group.into_iter().filter(|f| {
                f.name.to_lowercase().contains(&q)
                    || f.description.to_lowercase().contains(&q)
            }).collect()
        }
    };

    // Collect owned data
    struct FuncItem { name: String, starred: bool, selected: bool, signature: String }
    let mut func_items: Vec<FuncItem> = candidates.iter().map(|f| FuncItem {
        name: f.name.clone(),
        starred: f.starred,
        selected: ls.selected.as_deref() == Some(&f.name),
        signature: f.signature.clone(),
    }).collect();
    func_items.sort_by(|a, b| a.name.cmp(&b.name));

    let fn_btns: Vec<Element<Message>> = func_items.into_iter().map(|fi| {
        let color = if fi.selected { th::color::green() } else { Color::WHITE };
        let star = if fi.starred { "★" } else { "☆" };
        let name = fi.name.clone();
        let name2 = fi.name.clone();
        let name3 = fi.name.clone();
        let args = extract_param_names(&fi.signature);
        let display_name = if name2.len() > 22 {
            format!("{}…", &name2[..21])
        } else {
            name2.clone()
        };
        row![
            button(text(star).size(11)).on_press(Message::LibraryToggleStar(name)),
            button(text(display_name).size(12).color(color))
                .on_press(Message::LibrarySelect(Some(name2)))
                .width(Length::Fill),
            button(text("→").size(10))
                .on_press(Message::ComposerAddBlock(MainBlock::FunctionCall {
                    func_name: name3,
                    args,
                    assign_to: String::new(),
                    comment: String::new(),
                })),
        ]
        .spacing(2)
        .into()
    }).collect();

    let fn_empty = candidates.is_empty();
    let fn_panel = scrollable(
        if fn_empty {
            column![text("No functions.").size(12).color(th::color::text_dim())]
        } else {
            column(fn_btns).spacing(2)
        }
    )
    .width(200)
    .height(Length::Fill);

    // ── Detail panel ──────────────────────────────────────────────────────────
    let detail: Element<Message> = if ls.adding_new {
        edit_form(state, lib, None)
    } else if ls.edit_mode {
        if let Some(draft) = &ls.draft {
            edit_form(state, lib, Some(draft.clone()))
        } else if let Some(sel) = &ls.selected {
            if let Some(f) = lib.all().iter().find(|f| &f.name == sel) {
                edit_form(state, lib, Some(f.clone()))
            } else {
                text("Function not found.").into()
            }
        } else {
            text("No function selected.").into()
        }
    } else if let Some(sel) = &ls.selected {
        if let Some(f) = lib.all().iter().find(|f| &f.name == sel) {
            view_func(f, state.pending_library_insert_module.as_deref(), state.config.code_font_size)
        } else {
            text("Function not found.").into()
        }
    } else {
        text("Select a function").color(th::color::text_dim()).into()
    };

    let groups_cell = std::cell::RefCell::new(Some(group_panel.into()));
    let fns_cell = std::cell::RefCell::new(Some(fn_panel.into()));
    let detail_cell = std::cell::RefCell::new(Some(scrollable(detail).height(Length::Fill).into()));

    let grid: Element<Message> = pane_grid::PaneGrid::new(&state.library_panes, |_, pane, _| {
        let body: Element<Message> = match pane {
            LibraryPane::Groups => groups_cell.borrow_mut().take().unwrap(),
            LibraryPane::Functions => fns_cell.borrow_mut().take().unwrap(),
            LibraryPane::Detail => detail_cell.borrow_mut().take().unwrap(),
        };
        pane_grid::Content::new(body)
    })
    .on_resize(8, Message::LibraryPaneResized)
    .spacing(4)
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    column![
        toolbar,
        grid,
    ]
    .spacing(8)
    .padding(12)
    .into()
}

fn view_func<'a>(f: &'a FunctionTemplate, pending_module: Option<&str>, font_size: f32) -> Element<'a, Message> {
    let insert_btn: Option<Element<Message>> = pending_module.map(|m| {
        button(text(format!("→ Insert into {m}")).size(12))
            .on_press(Message::LibraryInsertToModule)
            .into()
    });
    let mut header_row = row![
        text(f.name.clone()).size(16),
        text(format!("({})", f.module)).size(12).color(th::color::text_dim()),
        Space::new().width(Length::Fill),
        button(text("Edit").size(12)).on_press(Message::LibraryEditMode(true)),
        button(text("Delete").size(12)).on_press(Message::LibraryDelete(f.name.clone())),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);
    if let Some(btn) = insert_btn {
        header_row = header_row.push(btn);
    }

    column![
        header_row,
        text(f.description.clone()).color(th::color::text()),
        {
            let tags_str = f.tags.join(", ");
            let tags_el: Element<Message> = if !tags_str.is_empty() {
                text(tags_str).size(11).color(th::color::green()).into()
            } else {
                Space::new().into()
            };
            tags_el
        },
        row![
            text("Prototype").size(12).color(th::color::cyan()),
            Space::new().width(Length::Fill),
            button(text("Copy").size(11)).on_press(Message::LibraryCopyCode(f.header_code.clone())),
        ]
        .align_y(iced::Alignment::Center),
        code_view(&f.header_code, (font_size - 1.0).max(8.0), None, None),
        row![
            text("Implementation").size(12).color(th::color::cyan()),
            Space::new().width(Length::Fill),
            button(text("Copy").size(11)).on_press(Message::LibraryCopyCode(f.impl_code.clone())),
        ]
        .align_y(iced::Alignment::Center),
        code_view(&f.impl_code, (font_size - 1.0).max(8.0), None, None),
        {
            let req_str = f.requires.join(", ");
            let requires_el: Element<Message> = if !req_str.is_empty() {
                text(format!("Requires: {req_str}")).size(11)
                    .color(th::color::yellow()).into()
            } else {
                Space::new().into()
            };
            requires_el
        },
    ]
    .spacing(8)
    .into()
}

fn edit_form<'a>(state: &'a AppState, _lib: &'a FunctionLibrary, draft: Option<FunctionTemplate>) -> Element<'a, Message> {
    let ls = &state.library_state;
    let draft_template = draft.as_ref().or(ls.draft.as_ref());

    let name_val = draft_template.map(|f| f.name.clone()).unwrap_or_default();
    let module_val = draft_template.map(|f| f.module.clone()).unwrap_or_default();
    let desc_val = draft_template.map(|f| f.description.clone()).unwrap_or_default();
    let sig_val = draft_template.map(|f| f.signature.clone()).unwrap_or_default();
    let tags_val = draft_template.map(|f| f.tags.join(", ")).unwrap_or_default();
    let _notes_val = draft_template.map(|f| f.notes.clone()).unwrap_or_default();

    let title = if ls.adding_new { "New Function" } else { "Edit Function" };
    let is_valid = !name_val.trim().is_empty() && !module_val.trim().is_empty();
    let mut save_btn = button(text("Save"));
    if is_valid {
        save_btn = save_btn.on_press(Message::LibrarySave(draft_template.cloned().unwrap_or_else(LibraryState::new_draft)));
    }

    column![
        text(title).size(16),
        row![
            text("Name:").width(100),
            text_input("function_name", &name_val)
                .on_input(|s| Message::LibraryDraftField(LibraryField::Name, s))
                .width(240),
        ]
        .spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Group:").width(100),
            text_input("group_name", &module_val)
                .on_input(|s| Message::LibraryDraftField(LibraryField::Module, s))
                .width(200),
        ]
        .spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Description:").width(100),
            text_input("description", &desc_val)
                .on_input(|s| Message::LibraryDraftField(LibraryField::Description, s))
                .width(320),
        ]
        .spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Tags:").width(100),
            text_input("tag1, tag2", &tags_val)
                .on_input(|s| Message::LibraryDraftField(LibraryField::Tags, s))
                .width(280),
        ]
        .spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Signature:").width(100),
            text_input("return_type name(type param)", &sig_val)
                .on_input(|s| Message::LibraryDraftField(LibraryField::Signature, s))
                .width(360),
        ]
        .spacing(8).align_y(iced::Alignment::Center),
        text("Header (.h):").size(12).color(th::color::cyan()),
        text_editor(&ls.header_editor)
            .highlight("c", iced::highlighter::Theme::Base16Mocha)
            .on_action(Message::LibraryHeaderEditAction)
            .font(iced::Font::MONOSPACE)
            .size((state.config.code_font_size - 1.0).max(8.0))
            .height(120),
        text("Implementation (.c):").size(12).color(th::color::cyan()),
        text_editor(&ls.impl_editor)
            .highlight("c", iced::highlighter::Theme::Base16Mocha)
            .on_action(Message::LibraryImplEditAction)
            .font(iced::Font::MONOSPACE)
            .size((state.config.code_font_size - 1.0).max(8.0))
            .height(220),
        row![
            save_btn,
            button(text("Cancel")).on_press(Message::LibraryEditMode(false)),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .into()
}

fn extract_param_names(sig: &str) -> Vec<String> {
    let open = sig.find('(').unwrap_or(sig.len());
    let close = sig.rfind(')').unwrap_or(sig.len());
    if open >= close { return Vec::new(); }
    let params = &sig[open + 1..close];
    if params.trim() == "void" || params.trim().is_empty() { return Vec::new(); }
    params.split(',').filter_map(|param| {
        let param = param.trim();
        let name = param.split_whitespace().last()?;
        let name = name.trim_start_matches('*');
        let name = name.split('[').next().unwrap_or(name);
        if name.is_empty() || name == "void" { None }
        else { Some(name.to_string()) }
    }).collect()
}

fn parse_sig(sig: &str) -> (String, Vec<(String, String)>) {
    let sig = sig.trim().trim_end_matches(';').trim();
    let Some(paren) = sig.find('(') else {
        return (String::new(), Vec::new());
    };
    let before = sig[..paren].trim();
    let inner = sig[paren + 1..].trim_end_matches(')').trim();

    let return_type = if let Some(pos) = before.rfind(|c: char| c.is_whitespace()) {
        before[..pos].trim().to_string()
    } else {
        String::new()
    };

    let params = if inner.is_empty() || inner == "void" {
        Vec::new()
    } else {
        inner.split(',').filter_map(|param| {
            let param = param.trim();
            if param.is_empty() { return None; }
            if let Some(sp) = param.rfind(|c: char| c.is_whitespace()) {
                let ptype = param[..sp].trim().to_string();
                let pname = param[sp..].trim().to_string();
                Some((ptype, pname))
            } else {
                Some((param.to_string(), String::new()))
            }
        }).collect()
    };
    (return_type, params)
}
