use std::collections::HashSet;
use std::path::{Path, PathBuf};

use egui::{Color32, RichText, ScrollArea, SidePanel, CentralPanel, Ui};
use newc_core::{
    function_lib::FunctionLibrary,
    sync::{ExtractedFunction, extract_function_implementations, extract_signatures},
};

#[derive(Default)]
pub struct ModuleDetailState {
    pub selected_func: Option<String>,
    pub edit_mode: bool,
    pub edit_buf: String,
    pub show_header_editor: bool,
    // Dead-code check
    pub unreachable_funcs: Vec<String>,
    pub check_ran: bool,
    // Prototype mismatch
    pub proto_mismatch: Option<String>,
    // Call tree
    pub show_call_tree: bool,
    pub call_tree_lines: Vec<String>,
}

pub enum ModuleDetailAction {
    None,
    GoBack,
    SaveFunction { name: String, new_impl: String },
    DeleteFunction(String),
    AddFromLibrary,
    OpenHeaderEditor,
    SyncNow,
    AutoFixInclude(String),
}

pub fn show(
    ui: &mut Ui,
    module_name: &str,
    src_path: &PathBuf,
    project_root: &Path,
    state: &mut ModuleDetailState,
    lib: &FunctionLibrary,
) -> ModuleDetailAction {
    let mut action = ModuleDetailAction::None;

    // Load source content + functions
    let src_content = src_path.exists()
        .then(|| std::fs::read_to_string(src_path).unwrap_or_default())
        .unwrap_or_default();
    let funcs: Vec<ExtractedFunction> = extract_function_implementations(&src_content);

    // Header row
    ui.horizontal(|ui| {
        if ui.button("← Project").clicked() {
            action = ModuleDetailAction::GoBack;
        }
        ui.heading(format!("Module: {module_name}"));
        ui.label(
            RichText::new(src_path.display().to_string())
                .small()
                .color(Color32::GRAY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Edit Header (.h)").clicked() {
                action = ModuleDetailAction::OpenHeaderEditor;
            }
            if ui.button("Add from Library").clicked() {
                action = ModuleDetailAction::AddFromLibrary;
            }
        });
    });

    // ── Missing include detection strip ──────────────────────────────────────
    let missing_includes = compute_missing_includes(&src_content, lib);
    if !missing_includes.is_empty() {
        egui::Frame::new()
            .fill(Color32::from_rgb(80, 60, 0))
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                for warn in &missing_includes {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("⚠ {warn}"))
                                .small()
                                .color(Color32::from_rgb(255, 210, 60)),
                        );
                        let header = warn
                            .split_whitespace()
                            .find(|w| w.ends_with(".h"))
                            .unwrap_or("")
                            .trim_end_matches('.')
                            .to_string();
                        if !header.is_empty() && ui.small_button("Auto-fix").clicked() {
                            action = ModuleDetailAction::AutoFixInclude(header);
                        }
                    });
                }
            });
    }

    // ── Prototype mismatch banner ─────────────────────────────────────────────
    if let Some(msg) = &state.proto_mismatch.clone() {
        egui::Frame::new()
            .fill(Color32::from_rgb(80, 50, 0))
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("⚠ {msg}"))
                            .small()
                            .color(Color32::from_rgb(255, 200, 60)),
                    );
                    if ui.small_button("Sync now").clicked() {
                        action = ModuleDetailAction::SyncNow;
                        state.proto_mismatch = None;
                    }
                    if ui.small_button("Dismiss").clicked() {
                        state.proto_mismatch = None;
                    }
                });
            });
    }

    ui.separator();

    // Left: function list
    SidePanel::left("mod_detail_list")
        .min_width(180.0)
        .max_width(260.0)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Functions").strong());
                let btn_text = if state.check_ran { "Re-check" } else { "Dead-code" };
                if ui.small_button(btn_text).on_hover_text("Run BFS reachability check").clicked() {
                    if let Ok(unreachable) = newc_core::analysis::check(project_root) {
                        state.unreachable_funcs = unreachable.iter().map(|f| f.name.clone()).collect();
                        state.check_ran = true;
                    }
                }
            });
            ui.separator();
            ScrollArea::vertical().id_salt("mod_fn_list").show(ui, |ui| {
                for func in &funcs {
                    let sel = state.selected_func.as_deref() == Some(&func.name);
                    let is_unreachable = state.unreachable_funcs.contains(&func.name);
                    let label = if is_unreachable {
                        RichText::new(format!("⚠ {}", func.name))
                            .color(Color32::from_rgb(255, 140, 0))
                    } else {
                        RichText::new(&func.name)
                    };
                    if ui.selectable_label(sel, label).clicked() && !sel {
                        state.selected_func = Some(func.name.clone());
                        state.edit_mode = false;
                        state.edit_buf.clear();
                        state.show_call_tree = false;
                        state.call_tree_lines.clear();
                    }
                }
                if funcs.is_empty() {
                    ui.label(RichText::new("No functions.").color(Color32::GRAY).small());
                }
            });
        });

    // Right: function detail / editor
    CentralPanel::default().show_inside(ui, |ui| {
        let Some(sel_name) = &state.selected_func.clone() else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select a function").color(Color32::GRAY));
            });
            return;
        };

        let Some(func) = funcs.iter().find(|f| &f.name == sel_name) else {
            ui.label("Function not found in source.");
            return;
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(&func.name).strong().monospace());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !state.edit_mode {
                    if ui.button("Edit").clicked() {
                        state.edit_buf = format!("{}\n{}", func.signature, func.body);
                        state.edit_mode = true;
                        state.show_call_tree = false;
                    }
                    if ui.button("Delete").on_hover_text("Remove this function").clicked() {
                        action = ModuleDetailAction::DeleteFunction(func.name.clone());
                        state.selected_func = None;
                    }
                    let tree_label = if state.show_call_tree { "Hide Tree" } else { "Call Tree" };
                    if ui.button(tree_label).on_hover_text("Show function call dependency tree").clicked() {
                        if !state.show_call_tree {
                            state.call_tree_lines = build_call_tree(
                                &func.name,
                                project_root,
                                &funcs,
                                0,
                                &mut HashSet::new(),
                            );
                        }
                        state.show_call_tree = !state.show_call_tree;
                    }
                }
            });
        });

        if !func.comment.is_empty() {
            ui.collapsing("Comment block", |ui| {
                ui.label(RichText::new(&func.comment).monospace().small().color(Color32::LIGHT_GRAY));
            });
        }

        ui.separator();

        if state.edit_mode {
            let ctrl_s = ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl);

            ui.label(RichText::new("Editing implementation: (Ctrl+S to save)").strong());
            ScrollArea::vertical().id_salt("mod_edit_scroll").max_height(360.0).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut state.edit_buf)
                        .code_editor()
                        .desired_rows(18)
                        .desired_width(f32::INFINITY),
                );
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save (Ctrl+S)").clicked() || ctrl_s {
                    action = ModuleDetailAction::SaveFunction {
                        name: func.name.clone(),
                        new_impl: state.edit_buf.clone(),
                    };
                    state.edit_mode = false;
                }
                if ui.button("Cancel").clicked() {
                    state.edit_mode = false;
                    state.edit_buf.clear();
                }
            });
        } else {
            ui.label(RichText::new("Signature:").strong());
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                ui.label(RichText::new(&func.signature).monospace());
            });
            ui.add_space(4.0);

            // Call tree panel
            if state.show_call_tree && !state.call_tree_lines.is_empty() {
                ui.label(RichText::new("Call Tree:").strong());
                egui::Frame::dark_canvas(ui.style())
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .id_salt("call_tree_scroll")
                            .max_height(140.0)
                            .show(ui, |ui| {
                                for line in &state.call_tree_lines {
                                    ui.label(RichText::new(line).monospace().size(11.0));
                                }
                            });
                    });
                ui.add_space(4.0);
            }

            ui.label(RichText::new("Body:").strong());
            ScrollArea::vertical().id_salt("mod_body_scroll").max_height(340.0).show(ui, |ui| {
                egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(&func.body).monospace().size(12.0),
                        )
                        .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });
            });
        }
    });

    action
}

// ── Missing include detection ─────────────────────────────────────────────────

fn compute_missing_includes(src_content: &str, lib: &FunctionLibrary) -> Vec<String> {
    let mut warnings = Vec::new();

    // Build map: func_name → module
    let func_to_module: Vec<(&str, &str)> = lib
        .all()
        .iter()
        .map(|f| (f.name.as_str(), f.module.as_str()))
        .collect();

    // Collect which modules are already included
    let mut included_modules: HashSet<String> = HashSet::new();
    for line in src_content.lines().take(30) {
        let t = line.trim();
        if t.starts_with("#include") && t.contains('"') {
            if let Some(start) = t.find('"') {
                if let Some(end) = t[start + 1..].find('"') {
                    let hdr = &t[start + 1..start + 1 + end];
                    let module = hdr.trim_end_matches(".h");
                    included_modules.insert(module.to_string());
                }
            }
        }
    }

    // Check each known function call
    let mut warned_modules: HashSet<String> = HashSet::new();
    for (fname, module) in &func_to_module {
        if included_modules.contains(*module) || warned_modules.contains(*module) {
            continue;
        }
        if src_content.contains(&format!("{fname}(")) {
            warned_modules.insert(module.to_string());
            warnings.push(format!(
                "{fname}() used but {module}.h not included — add #include \"{module}.h\""
            ));
        }
    }

    warnings
}

// ── Prototype mismatch check ──────────────────────────────────────────────────

pub fn check_proto_mismatch(src_path: &Path, module_name: &str) -> Option<String> {
    let hdr_path = src_path
        .parent()?
        .parent()?
        .join("include")
        .join(format!("{module_name}.h"));

    let src_content = std::fs::read_to_string(src_path).ok()?;
    let hdr_content = std::fs::read_to_string(&hdr_path).ok()?;

    let src_sigs: HashSet<String> = extract_function_implementations(&src_content)
        .into_iter()
        .map(|f| normalise_sig(&f.signature))
        .collect();

    let hdr_sigs: HashSet<String> = extract_signatures(&hdr_content)
        .into_iter()
        .map(|s| normalise_sig(&s))
        .collect();

    let mismatched: Vec<&String> = src_sigs.iter().filter(|s| !hdr_sigs.contains(*s)).collect();
    if mismatched.is_empty() {
        return None;
    }
    Some(format!(
        "Prototype mismatch: {} signature(s) in .c not matching .h",
        mismatched.len()
    ))
}

fn normalise_sig(sig: &str) -> String {
    sig.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ── Call dependency tree ──────────────────────────────────────────────────────

fn build_call_tree(
    fname: &str,
    project_root: &Path,
    local_funcs: &[ExtractedFunction],
    depth: usize,
    visited: &mut HashSet<String>,
) -> Vec<String> {
    let indent = "  ".repeat(depth);
    let mut lines = vec![format!("{indent}{fname}()")];

    if depth >= 5 || visited.contains(fname) {
        if depth >= 5 {
            lines.push(format!("{indent}  (max depth)"));
        }
        return lines;
    }
    visited.insert(fname.to_string());

    // Find the body of fname in local funcs or in all src files
    let body = local_funcs
        .iter()
        .find(|f| f.name == fname)
        .map(|f| f.body.clone())
        .or_else(|| find_body_in_project(fname, project_root));

    let Some(body) = body else { return lines };

    // Find calls to known functions in body
    let all_names: Vec<String> = collect_all_func_names(project_root, local_funcs);
    for callee in &all_names {
        if callee == fname { continue; }
        if body.contains(&format!("{callee}(")) {
            let sub = build_call_tree(callee, project_root, local_funcs, depth + 1, visited);
            lines.extend(sub);
        }
    }

    lines
}

fn collect_all_func_names(project_root: &Path, local: &[ExtractedFunction]) -> Vec<String> {
    let mut names: Vec<String> = local.iter().map(|f| f.name.clone()).collect();
    let src_dir = project_root.join("src");
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) == Some("c") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for f in extract_function_implementations(&content) {
                        if !names.contains(&f.name) {
                            names.push(f.name);
                        }
                    }
                }
            }
        }
    }
    names
}

fn find_body_in_project(fname: &str, project_root: &Path) -> Option<String> {
    let src_dir = project_root.join("src");
    let entries = std::fs::read_dir(&src_dir).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) == Some("c") {
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            if let Some(f) = extract_function_implementations(&content)
                .into_iter()
                .find(|f| f.name == fname)
            {
                return Some(f.body);
            }
        }
    }
    None
}
