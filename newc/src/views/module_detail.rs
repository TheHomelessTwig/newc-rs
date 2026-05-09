use egui::{Color32, RichText, ScrollArea, SidePanel, CentralPanel, Ui};
use newc_core::{function_lib::FunctionLibrary, sync::{ExtractedFunction, extract_function_implementations}};
use std::path::PathBuf;

#[derive(Default)]
pub struct ModuleDetailState {
    pub selected_func: Option<String>,
    pub edit_mode: bool,
    pub edit_buf: String,
    pub show_header_editor: bool,
}

pub enum ModuleDetailAction {
    None,
    GoBack,
    SaveFunction { name: String, new_impl: String },
    DeleteFunction(String),
    AddFromLibrary,
    OpenHeaderEditor,
}

pub fn show(
    ui: &mut Ui,
    module_name: &str,
    src_path: &PathBuf,
    state: &mut ModuleDetailState,
    _lib: &FunctionLibrary,
) -> ModuleDetailAction {
    let mut action = ModuleDetailAction::None;

    // Load functions from source file
    let funcs: Vec<ExtractedFunction> = if src_path.exists() {
        let content = std::fs::read_to_string(src_path).unwrap_or_default();
        extract_function_implementations(&content)
    } else {
        Vec::new()
    };

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
    ui.separator();

    // Left: function list
    SidePanel::left("mod_detail_list")
        .min_width(180.0)
        .max_width(240.0)
        .show_inside(ui, |ui| {
            ui.label(RichText::new("Functions").strong());
            ui.separator();
            ScrollArea::vertical().id_salt("mod_fn_list").show(ui, |ui| {
                for func in &funcs {
                    let sel = state.selected_func.as_deref() == Some(&func.name);
                    if ui.selectable_label(sel, &func.name).clicked() && !sel {
                        state.selected_func = Some(func.name.clone());
                        state.edit_mode = false;
                        state.edit_buf.clear();
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
                        // Load current impl into buffer
                        state.edit_buf =
                            format!("{}\n{}", func.signature, func.body);
                        state.edit_mode = true;
                    }
                    if ui.button("Delete").on_hover_text("Remove this function").clicked() {
                        action = ModuleDetailAction::DeleteFunction(func.name.clone());
                        state.selected_func = None;
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
            // Ctrl+S to save
            let ctrl_s = ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl);

            ui.label(RichText::new("Editing implementation: (Ctrl+S to save)").strong());
            ScrollArea::vertical().id_salt("mod_edit_scroll").show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut state.edit_buf)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(f32::INFINITY),
                );
            });
            ui.separator();
            let save = ctrl_s;
            ui.horizontal(|ui| {
                if ui.button("Save (Ctrl+S)").clicked() || save {
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
            ui.label(RichText::new("Body:").strong());
            ScrollArea::vertical().id_salt("mod_body_scroll").max_height(400.0).show(ui, |ui| {
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
