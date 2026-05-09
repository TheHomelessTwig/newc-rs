use egui::{Color32, RichText, ScrollArea, SidePanel, CentralPanel, Ui};
use newc_core::function_lib::{FunctionLibrary, FunctionTemplate};

#[derive(Default, Clone)]
pub struct LibraryState {
    pub selected: Option<String>,
    pub search: String,
    pub edit_mode: bool,
    pub draft: Option<FunctionTemplate>,
    pub adding_new: bool,
    /// Which group is selected in the left sidebar (None = show all)
    pub active_group: Option<String>,
    /// Rename input for the group rename dialog
    pub rename_input: String,
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
}

pub enum LibraryAction {
    None,
    Save(FunctionTemplate),
    Delete(String),
    UpdateNotes { name: String, notes: String },
    ToggleStar(String),
    OpenImport,
    CreateGroup { name: String, description: String },
    RenameGroup { old: String, new: String },
    DeleteGroup { name: String, cascade: bool },
}

pub fn show(ui: &mut Ui, lib: &FunctionLibrary, state: &mut LibraryState) -> LibraryAction {
    let mut action = LibraryAction::None;

    // ── Top toolbar ──────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.heading("Function Library");
        ui.separator();
        ui.text_edit_singleline(&mut state.search)
            .labelled_by(ui.label("Search:").id);
        if ui.button("+ New Function").clicked() {
            state.adding_new = true;
            state.edit_mode = true;
            let mut draft = LibraryState::new_draft();
            // Pre-fill module from active group
            if let Some(g) = &state.active_group {
                draft.module = g.clone();
            }
            state.draft = Some(draft);
            state.selected = None;
        }
        if ui.button("Import from .c…").clicked() {
            action = LibraryAction::OpenImport;
        }
    });
    ui.separator();

    // ── New-function form (full-width) ───────────────────────────────────────
    if state.adding_new {
        if state.draft.is_none() {
            state.draft = Some(LibraryState::new_draft());
        }
        let mut draft = state.draft.take().unwrap();
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            ui.heading("New Function");
            show_edit_form(ui, &mut draft, lib);
        });
        let valid = !draft.name.trim().is_empty() && !draft.module.trim().is_empty();
        ui.horizontal(|ui| {
            if ui.add_enabled(valid, egui::Button::new("Save")).clicked() {
                action = LibraryAction::Save(draft.clone());
                state.adding_new = false;
                state.edit_mode = false;
                state.selected = Some(draft.name.clone());
            } else {
                state.draft = Some(draft);
            }
            if ui.button("Cancel").clicked() {
                state.adding_new = false;
                state.edit_mode = false;
                state.draft = None;
            }
        });
        ui.separator();
        return action;
    }

    // ── Left panel: Groups ───────────────────────────────────────────────────
    SidePanel::left("lib_groups")
        .min_width(150.0)
        .max_width(220.0)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Groups").strong());
                if ui.small_button("+").on_hover_text("New group").clicked() {
                    // Signal via a special "new_group" selected state; app.rs shows modal
                    action = LibraryAction::CreateGroup {
                        name: "__OPEN_DIALOG__".to_string(),
                        description: String::new(),
                    };
                }
            });
            ui.separator();

            ScrollArea::vertical().id_salt("groups_scroll").show(ui, |ui| {
                // "All" entry
                let all_sel = state.active_group.is_none();
                if ui.selectable_label(all_sel, "All").clicked() {
                    state.active_group = None;
                    state.selected = None;
                }

                // "Starred" filter
                let starred_count = lib.all().iter().filter(|f| f.starred).count();
                let starred_sel = state.active_group.as_deref() == Some("__starred__");
                if ui.selectable_label(starred_sel, format!("★ Starred ({})", starred_count)).clicked() {
                    state.active_group = Some("__starred__".to_string());
                    state.selected = None;
                }
                ui.separator();

                for group in &lib.groups {
                    let count = lib.by_module(&group.name).len();
                    let is_sel = state.active_group.as_deref() == Some(&group.name);
                    ui.horizontal(|ui| {
                        let label =
                            format!("{} ({})", group.name, count);
                        if ui.selectable_label(is_sel, &label).clicked() {
                            state.active_group = Some(group.name.clone());
                            state.selected = None;
                        }
                        if !group.builtin {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("…")
                                        .on_hover_text("Rename / delete")
                                        .clicked()
                                    {
                                        // Signal app.rs to open the group action modal
                                        state.rename_input = group.name.clone();
                                        action = LibraryAction::RenameGroup {
                                            old: "__OPEN_DIALOG__".to_string(),
                                            new: group.name.clone(),
                                        };
                                    }
                                },
                            );
                        }
                    });
                }
            });
        });

    // ── Right: function list + detail ────────────────────────────────────────
    // Build candidate list filtered by active group and search
    let candidates: Vec<_> = {
        let by_group: Vec<_> = match state.active_group.as_deref() {
            Some("__starred__") => lib.all().iter().filter(|f| f.starred).collect(),
            Some(g) => lib.all().iter().filter(|f| f.module == g).collect(),
            None => lib.all().iter().collect(),
        };
        if state.search.is_empty() {
            by_group
        } else {
            let q = state.search.to_lowercase();
            by_group
                .into_iter()
                .filter(|f| {
                    f.name.to_lowercase().contains(&q)
                        || f.description.to_lowercase().contains(&q)
                        || f.tags.iter().any(|t| t.to_lowercase().contains(&q))
                })
                .collect()
        }
    };

    // Sub-split: function list (inner left) + detail (inner right)
    SidePanel::left("lib_list")
        .min_width(180.0)
        .max_width(260.0)
        .show_inside(ui, |ui| {
            ScrollArea::vertical().id_salt("lib_list_scroll").show(ui, |ui| {
                let mut modules: Vec<String> =
                    candidates.iter().map(|f| f.module.clone()).collect();
                modules.sort();
                modules.dedup();

                for module in &modules {
                    ui.collapsing(module.as_str(), |ui| {
                        for func in candidates.iter().filter(|f| &f.module == module) {
                            let selected = state.selected.as_deref() == Some(&func.name);
                            ui.horizontal(|ui| {
                                let star = if func.starred { "★" } else { "☆" };
                                if ui.small_button(star).on_hover_text("Toggle favourite").clicked() {
                                    action = LibraryAction::ToggleStar(func.name.clone());
                                }
                                if ui.selectable_label(selected, &func.name).clicked() {
                                    if state.selected.as_deref() != Some(&func.name) {
                                        state.selected = Some(func.name.clone());
                                        state.edit_mode = false;
                                        state.draft = None;
                                    }
                                }
                            });
                        }
                    });
                }

                if candidates.is_empty() {
                    ui.label(
                        RichText::new("No functions in this group.")
                            .color(Color32::GRAY)
                            .small(),
                    );
                }
            });
        });

    // ── Detail / edit pane ───────────────────────────────────────────────────
    CentralPanel::default().show_inside(ui, |ui| {
        let Some(sel_name) = &state.selected.clone() else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select a function").color(Color32::GRAY));
            });
            return;
        };

        let Some(func) = lib.all().iter().find(|f| &f.name == sel_name) else {
            ui.label("Function not found.");
            return;
        };

        if state.edit_mode {
            if state.draft.is_none() {
                state.draft = Some(func.clone());
            }
            let mut draft = state.draft.take().unwrap();
            ScrollArea::vertical().id_salt("lib_edit").show(ui, |ui| {
                show_edit_form(ui, &mut draft, lib);
            });
            ui.separator();
            let valid = !draft.name.trim().is_empty() && !draft.module.trim().is_empty();
            let mut saved = false;
            ui.horizontal(|ui| {
                if ui.add_enabled(valid, egui::Button::new("Save")).clicked() {
                    action = LibraryAction::Save(draft.clone());
                    state.selected = Some(draft.name.clone());
                    state.edit_mode = false;
                    saved = true;
                }
                if ui.button("Cancel").clicked() {
                    state.edit_mode = false;
                }
            });
            if !saved && state.edit_mode {
                state.draft = Some(draft);
            }
        } else {
            ScrollArea::vertical().id_salt("lib_view").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(&func.name);
                    ui.label(
                        RichText::new(format!("({})", func.module))
                            .color(Color32::GRAY)
                            .small(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Edit").clicked() {
                            state.edit_mode = true;
                        }
                        if ui.button("Delete").on_hover_text("Remove from library").clicked() {
                            action = LibraryAction::Delete(func.name.clone());
                            state.selected = None;
                        }
                    });
                });

                ui.label(RichText::new(&func.description).color(Color32::LIGHT_GRAY));

                if !func.tags.is_empty() {
                    ui.horizontal(|ui| {
                        for tag in &func.tags {
                            ui.label(
                                RichText::new(tag)
                                    .small()
                                    .color(Color32::from_rgb(100, 200, 150)),
                            );
                        }
                    });
                }

                if !func.requires.is_empty() {
                    ui.label(
                        RichText::new(format!("Requires: {}", func.requires.join(", ")))
                            .small()
                            .color(Color32::from_rgb(200, 170, 100)),
                    );
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Prototype (.h)").strong());
                code_block(ui, &func.header_code);

                ui.add_space(8.0);
                ui.label(RichText::new("Implementation (.c)").strong());
                code_block(ui, &func.impl_code);

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("Notes / Comments").strong());
                ui.label(
                    RichText::new("Personal notes — not written to C files.")
                        .small()
                        .color(Color32::GRAY),
                );
                let mut notes = func.notes.clone();
                let resp = ui.add(
                    egui::TextEdit::multiline(&mut notes)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)
                        .hint_text("Add notes, gotchas, usage examples…"),
                );
                if resp.changed() {
                    action = LibraryAction::UpdateNotes {
                        name: func.name.clone(),
                        notes,
                    };
                }
            });
        }
    });

    action
}

fn show_edit_form(ui: &mut Ui, draft: &mut FunctionTemplate, lib: &FunctionLibrary) {
    egui::Grid::new("func_edit_grid")
        .num_columns(2)
        .min_col_width(80.0)
        .show(ui, |ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut draft.name);
            ui.end_row();

            // Group/module as a combo-box populated from known groups
            ui.label("Group:");
            egui::ComboBox::from_id_salt("func_module_combo")
                .selected_text(if draft.module.is_empty() { "— pick group —" } else { &draft.module })
                .show_ui(ui, |ui| {
                    for group in &lib.groups {
                        ui.selectable_value(&mut draft.module, group.name.clone(), &group.name);
                    }
                    ui.separator();
                    // Allow typing a new group name via a text field inside the combo
                    // (handled below — free-text fallback)
                });
            // Free-text override if they want a brand-new group name
            if !lib.groups.iter().any(|g| g.name == draft.module) && !draft.module.is_empty() {
                ui.label(
                    RichText::new(format!("New group: {}", draft.module))
                        .small()
                        .color(Color32::from_rgb(100, 200, 150)),
                );
            }
            ui.end_row();

            // Allow typing module name directly if desired
            ui.label("Group (type):");
            ui.text_edit_singleline(&mut draft.module);
            ui.end_row();

            ui.label("Description:");
            ui.text_edit_singleline(&mut draft.description);
            ui.end_row();

            ui.label("Signature:");
            ui.text_edit_singleline(&mut draft.signature);
            ui.end_row();

            ui.label("Tags:");
            let mut tags_str = draft.tags.join(", ");
            if ui.text_edit_singleline(&mut tags_str).changed() {
                draft.tags = tags_str
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
            ui.end_row();

            ui.label("Requires:");
            let mut req_str = draft.requires.join(", ");
            if ui.text_edit_singleline(&mut req_str).changed() {
                draft.requires = req_str
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.label(RichText::new("Prototype (.h):").strong());
    ui.add(
        egui::TextEdit::multiline(&mut draft.header_code)
            .code_editor()
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );

    ui.add_space(4.0);
    ui.label(RichText::new("Implementation (.c):").strong());
    ui.add(
        egui::TextEdit::multiline(&mut draft.impl_code)
            .code_editor()
            .desired_rows(12)
            .desired_width(f32::INFINITY),
    );

    ui.add_space(4.0);
    ui.label(RichText::new("Notes:").strong());
    ui.add(
        egui::TextEdit::multiline(&mut draft.notes)
            .desired_rows(3)
            .desired_width(f32::INFINITY)
            .hint_text("Personal notes…"),
    );
}

fn code_block(ui: &mut Ui, code: &str) {
    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
        ScrollArea::horizontal().id_salt(code.len()).show(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(code).monospace().size(12.0))
                    .wrap_mode(egui::TextWrapMode::Extend),
            );
        });
    });
}
