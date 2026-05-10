use egui::{Color32, RichText, ScrollArea, SidePanel, CentralPanel, Ui};
use newc_core::function_lib::{FunctionLibrary, FunctionTemplate, detect_requires};

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
    // ── Structured signature builder ──────────────────────────────────────────
    /// (type, name) pairs for the parameter builder
    pub draft_params: Vec<(String, String)>,
    pub draft_return_type: String,
    /// When true, user types the signature manually instead of builder
    pub draft_override_sig: bool,
    /// Guards one-time init of draft_params when entering edit mode
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

// ── Signature helpers ─────────────────────────────────────────────────────────

/// Best-effort parse of `return_type name(params)` → (return_type, [(type, name)])
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
        inner
            .split(',')
            .filter_map(|p| {
                let p = p.trim();
                if p.is_empty() { return None; }
                if let Some(sp) = p.rfind(|c: char| c.is_whitespace()) {
                    let mut ptype = p[..sp].trim().to_string();
                    let pname = p[sp..].trim().to_string();
                    // `int *ptr` → after rfind: ptype=`int`, pname=`*ptr`; move `*` to type
                    if pname.starts_with('*') {
                        ptype.push('*');
                        Some((ptype, pname.trim_start_matches('*').trim().to_string()))
                    } else {
                        Some((ptype, pname))
                    }
                } else {
                    Some((p.to_string(), String::new()))
                }
            })
            .collect()
    };
    (return_type, params)
}

fn build_sig(return_type: &str, name: &str, params: &[(String, String)]) -> String {
    let params_str = if params.is_empty() {
        "void".to_string()
    } else {
        params
            .iter()
            .map(|(t, n)| if n.is_empty() { t.clone() } else { format!("{t} {n}") })
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!("{return_type} {name}({params_str})")
}

fn build_skeleton(return_type: &str, name: &str, params: &[(String, String)]) -> (String, String) {
    let sig = build_sig(return_type, name, params);

    let input_docs = if params.is_empty() {
        " *     None\n".to_string()
    } else {
        params
            .iter()
            .map(|(t, n)| format!(" *     {} - ({})\n", if n.is_empty() { t } else { n }, t))
            .collect()
    };

    let return_doc = if return_type == "void" || return_type.is_empty() {
        " *     None\n"
    } else {
        " *     result\n"
    };

    let return_stmt = if return_type == "void" || return_type.is_empty() {
        String::new()
    } else {
        "\t/* TODO: compute and return result */\n".to_string()
    };

    let header = format!("{sig};\n");
    let impl_code = format!(
        "/*\n * Function:\n *     {name}\n * Input:\n{input_docs} * Output:\n{return_doc} * Algorithm:\n *     TODO\n */\n{sig}\n{{\n{return_stmt}}}\n"
    );
    (header, impl_code)
}

// ── Main show ─────────────────────────────────────────────────────────────────

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
            if let Some(g) = &state.active_group {
                draft.module = g.clone();
            }
            state.draft = Some(draft);
            state.selected = None;
            state.reset_builder();
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
            state.reset_builder();
        }
        let mut draft = state.draft.take().unwrap();
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            ui.heading("New Function");
            show_edit_form(ui, &mut draft, lib, state);
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
                    action = LibraryAction::CreateGroup {
                        name: "__OPEN_DIALOG__".to_string(),
                        description: String::new(),
                    };
                }
            });
            ui.separator();

            ScrollArea::vertical().id_salt("groups_scroll").show(ui, |ui| {
                let all_sel = state.active_group.is_none();
                if ui.selectable_label(all_sel, "All").clicked() {
                    state.active_group = None;
                    state.selected = None;
                }

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
                        let label = format!("{} ({})", group.name, count);
                        if ui.selectable_label(is_sel, &label).clicked() {
                            state.active_group = Some(group.name.clone());
                            state.selected = None;
                        }
                        if !group.builtin {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("…").on_hover_text("Rename / delete").clicked() {
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

    // ── Build candidate list ─────────────────────────────────────────────────
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

    // ── Function list (inner left) ───────────────────────────────────────────
    SidePanel::left("lib_list")
        .min_width(180.0)
        .max_width(260.0)
        .show_inside(ui, |ui| {
            ScrollArea::vertical().id_salt("lib_list_scroll").show(ui, |ui| {
                let show_flat = state.active_group.is_none();

                if show_flat {
                    // ── Flat alphabetical list (no group headers) ─────────────
                    let mut sorted = candidates.clone();
                    sorted.sort_by(|a, b| a.name.cmp(&b.name));
                    for func in sorted {
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
                } else {
                    // ── Grouped list (when a group filter is active) ──────────
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
                // Init builder from existing signature on first open
                let sig = func.signature.clone();
                state.init_params_from_sig(&sig);
                state.draft_override_sig = false;
            }
            let mut draft = state.draft.take().unwrap();
            ScrollArea::vertical().id_salt("lib_edit").show(ui, |ui| {
                show_edit_form(ui, &mut draft, lib, state);
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
                    state.draft_params_ready = false;
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
                            state.draft_params_ready = false; // trigger init on next frame
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

// ── Edit form ─────────────────────────────────────────────────────────────────

fn show_edit_form(
    ui: &mut Ui,
    draft: &mut FunctionTemplate,
    lib: &FunctionLibrary,
    state: &mut LibraryState,
) {
    // ── Metadata grid ────────────────────────────────────────────────────────
    egui::Grid::new("func_edit_grid")
        .num_columns(2)
        .min_col_width(80.0)
        .show(ui, |ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut draft.name);
            ui.end_row();

            ui.label("Group:");
            egui::ComboBox::from_id_salt("func_module_combo")
                .selected_text(if draft.module.is_empty() { "— pick group —" } else { &draft.module })
                .show_ui(ui, |ui| {
                    for group in &lib.groups {
                        ui.selectable_value(&mut draft.module, group.name.clone(), &group.name);
                    }
                });
            if !lib.groups.iter().any(|g| g.name == draft.module) && !draft.module.is_empty() {
                ui.label(
                    RichText::new(format!("New group: {}", draft.module))
                        .small()
                        .color(Color32::from_rgb(100, 200, 150)),
                );
            }
            ui.end_row();

            ui.label("Description:");
            ui.text_edit_singleline(&mut draft.description);
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
        });

    ui.add_space(8.0);
    ui.separator();

    // ── Signature builder ────────────────────────────────────────────────────
    ui.label(RichText::new("Signature Builder").strong());

    // Return type + function name row
    ui.horizontal(|ui| {
        ui.label("Return type:");
        ui.add(
            egui::TextEdit::singleline(&mut state.draft_return_type)
                .desired_width(80.0)
                .hint_text("void"),
        );
        ui.add_space(12.0);
        ui.label("Function name:");
        ui.add(
            egui::TextEdit::singleline(&mut draft.name)
                .desired_width(140.0)
                .hint_text("my_function"),
        );
    });

    // Parameters list
    ui.add_space(4.0);
    ui.label(RichText::new("Parameters:").small().color(Color32::GRAY));
    let mut to_remove: Option<usize> = None;
    for (i, (ptype, pname)) in state.draft_params.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            if ui.small_button("X").clicked() {
                to_remove = Some(i);
            }
            ui.add(egui::TextEdit::singleline(ptype).desired_width(90.0).hint_text("type"));
            ui.add(egui::TextEdit::singleline(pname).desired_width(90.0).hint_text("name"));
        });
    }
    if let Some(i) = to_remove {
        state.draft_params.remove(i);
    }
    if ui.small_button("+ Add Parameter").clicked() {
        state.draft_params.push((String::new(), String::new()));
    }

    // Live signature generation
    ui.add_space(4.0);
    let ret = state.draft_return_type.trim().to_string();
    let fn_name = draft.name.trim().to_string();
    if !fn_name.is_empty() && !ret.is_empty() {
        let gen_sig = build_sig(&ret, &fn_name, &state.draft_params);
        if !state.draft_override_sig {
            draft.signature = gen_sig.clone();
            draft.header_code = format!("{gen_sig};\n");
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("Generated:").small().color(Color32::GRAY));
            ui.label(RichText::new(&gen_sig).monospace().small().color(Color32::LIGHT_GRAY));
        });

        // Skeleton button — only fill if impl_code is empty
        if draft.impl_code.trim().is_empty() {
            if ui.small_button("Generate skeleton").on_hover_text("Fill header_code + impl_code from signature").clicked() {
                let (hdr, imp) = build_skeleton(&ret, &fn_name, &state.draft_params);
                draft.header_code = hdr;
                draft.impl_code = imp;
            }
        } else if ui.small_button("Regenerate skeleton").on_hover_text("Overwrite header_code + impl_code from signature").clicked() {
            let (hdr, imp) = build_skeleton(&ret, &fn_name, &state.draft_params);
            draft.header_code = hdr;
            draft.impl_code = imp;
        }
    }

    ui.checkbox(&mut state.draft_override_sig, "Override signature manually");
    if state.draft_override_sig {
        ui.horizontal(|ui| {
            ui.label("Signature:");
            ui.add(
                egui::TextEdit::singleline(&mut draft.signature)
                    .desired_width(f32::INFINITY)
                    .hint_text("return_type name(type param, ...)"),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Header (.h):");
            ui.add(
                egui::TextEdit::singleline(&mut draft.header_code)
                    .desired_width(f32::INFINITY)
                    .hint_text("return_type name(type param, ...);"),
            );
        });
    }

    ui.add_space(4.0);
    ui.separator();

    // ── Requires ─────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("Requires:").strong());
        if ui.small_button("Detect").on_hover_text("Scan implementation for stdlib and library calls").clicked() {
            let detected = detect_requires(&draft.impl_code, lib);
            for r in detected {
                if !draft.requires.contains(&r) {
                    draft.requires.push(r);
                }
            }
        }
        if ui.small_button("Clear").clicked() {
            draft.requires.clear();
        }
    });
    ui.label(
        RichText::new("System headers (stdio.h, math.h…) and library function names this function depends on.")
            .small()
            .color(Color32::GRAY),
    );
    let mut req_str = draft.requires.join(", ");
    if ui.add(
        egui::TextEdit::singleline(&mut req_str)
            .desired_width(f32::INFINITY)
            .hint_text("stdio.h, math.h, my_helper_fn, …"),
    ).changed() {
        draft.requires = req_str
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
    }

    ui.add_space(4.0);
    ui.separator();

    // ── Code blocks ──────────────────────────────────────────────────────────
    ui.label(RichText::new("Prototype (.h):").strong());
    ui.add(
        egui::TextEdit::multiline(&mut draft.header_code)
            .code_editor()
            .desired_rows(2)
            .desired_width(f32::INFINITY),
    );

    ui.add_space(4.0);
    ui.label(RichText::new("Implementation (.c):").strong());
    ui.add(
        egui::TextEdit::multiline(&mut draft.impl_code)
            .code_editor()
            .desired_rows(14)
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
