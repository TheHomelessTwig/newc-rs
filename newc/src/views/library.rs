use iced::widget::{button, column, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length};
use newc_core::function_lib::{FunctionLibrary, FunctionTemplate};
use newc_core::main_builder::MainBlock;

use crate::state::{AppState, LibraryField, Message};

#[derive(Default, Clone)]
pub struct LibraryState {
    pub selected: Option<String>,
    pub search: String,
    pub edit_mode: bool,
    pub draft: Option<FunctionTemplate>,
    pub adding_new: bool,
    pub active_group: Option<String>,
    pub rename_input: String,
    pub draft_params: Vec<(String, String)>,
    pub draft_return_type: String,
    pub draft_override_sig: bool,
    pub draft_params_ready: bool,
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

#[derive(Debug, Clone)]
pub enum LibraryAction {
    None,
    Save(FunctionTemplate),
    Delete(String),
    UpdateNotes { name: String, notes: String },
    ToggleStar(String),
    OpenImport,
    CreateGroup { name: String, desc: String },
    RenameGroup { old: String, new: String },
    DeleteGroup { name: String, cascade: bool },
}

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
        let color = if fi.selected { Color::from_rgb(0.663, 0.863, 0.463) } else { Color::WHITE };
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
            column![text("No functions.").size(12).color(Color::from_rgb(0.5, 0.5, 0.5))]
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
            view_func(f)
        } else {
            text("Function not found.").into()
        }
    } else {
        text("Select a function").color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    };

    column![
        toolbar,
        row![
            group_panel,
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

fn view_func(f: &FunctionTemplate) -> Element<'_, Message> {
    column![
        row![
            text(f.name.clone()).size(16),
            text(format!("({})", f.module)).size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
            Space::new().width(Length::Fill),
            button(text("Edit").size(12)).on_press(Message::LibraryEditMode(true)),
            button(text("Delete").size(12)).on_press(Message::LibraryDelete(f.name.clone())),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
        text(f.description.clone()).color(Color::from_rgb(0.85, 0.85, 0.85)),
        {
            let tags_str = f.tags.join(", ");
            let el: Element<Message> = if !tags_str.is_empty() {
                text(tags_str).size(11).color(Color::from_rgb(0.392, 0.784, 0.588)).into()
            } else {
                Space::new().into()
            };
            el
        },
        text("Prototype").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
        text(f.header_code.clone()).font(iced::Font::MONOSPACE).size(12),
        text("Implementation").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
        text(f.impl_code.clone()).font(iced::Font::MONOSPACE).size(12),
        {
            let req_str = f.requires.join(", ");
            let el: Element<Message> = if !req_str.is_empty() {
                text(format!("Requires: {req_str}")).size(11)
                    .color(Color::from_rgb(0.784, 0.667, 0.392)).into()
            } else {
                Space::new().into()
            };
            el
        },
    ]
    .spacing(8)
    .into()
}

fn edit_form<'a>(state: &'a AppState, _lib: &'a FunctionLibrary, draft: Option<FunctionTemplate>) -> Element<'a, Message> {
    let ls = &state.library_state;
    let d = draft.as_ref().or(ls.draft.as_ref());

    let name_val = d.map(|f| f.name.clone()).unwrap_or_default();
    let module_val = d.map(|f| f.module.clone()).unwrap_or_default();
    let desc_val = d.map(|f| f.description.clone()).unwrap_or_default();
    let sig_val = d.map(|f| f.signature.clone()).unwrap_or_default();
    let tags_val = d.map(|f| f.tags.join(", ")).unwrap_or_default();
    let header_val = d.map(|f| f.header_code.clone()).unwrap_or_default();
    let impl_val = d.map(|f| f.impl_code.clone()).unwrap_or_default();
    let _notes_val = d.map(|f| f.notes.clone()).unwrap_or_default();

    let title = if ls.adding_new { "New Function" } else { "Edit Function" };
    let is_valid = !name_val.trim().is_empty() && !module_val.trim().is_empty();
    let mut save_btn = button(text("Save"));
    if is_valid {
        save_btn = save_btn.on_press(Message::LibrarySave(d.cloned().unwrap_or_else(LibraryState::new_draft)));
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
        text("Header (.h):").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
        text(header_val.clone()).font(iced::Font::MONOSPACE).size(12),
        text("Implementation (.c):").size(12).color(Color::from_rgb(0.471, 0.863, 0.910)),
        text(impl_val.clone()).font(iced::Font::MONOSPACE).size(12),
        text("(Full code editor integration coming in next iteration)")
            .size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
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
    params.split(',').filter_map(|p| {
        let p = p.trim();
        let name = p.split_whitespace().last()?;
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
        inner.split(',').filter_map(|p| {
            let p = p.trim();
            if p.is_empty() { return None; }
            if let Some(sp) = p.rfind(|c: char| c.is_whitespace()) {
                let ptype = p[..sp].trim().to_string();
                let pname = p[sp..].trim().to_string();
                Some((ptype, pname))
            } else {
                Some((p.to_string(), String::new()))
            }
        }).collect()
    };
    (return_type, params)
}
