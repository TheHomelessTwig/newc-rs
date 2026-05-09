use egui::{Color32, RichText, ScrollArea, Ui};
use newc_core::function_lib::FunctionTemplate;
use newc_core::sync::ExtractedFunction;

#[derive(Default)]
pub struct ImportState {
    pub extracted: Vec<ExtractedFunction>,
    pub selected: Vec<bool>,
    pub target_module: String,
    pub path_label: String,
}

pub enum ImportAction {
    None,
    PickFile,
    Import(Vec<FunctionTemplate>),
    Cancel,
}

pub fn show(ui: &mut Ui, state: &mut ImportState) -> ImportAction {
    let mut action = ImportAction::None;

    ui.horizontal(|ui| {
        ui.heading("Import from .c file");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
                action = ImportAction::Cancel;
            }
        });
    });
    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Browse .c file…").clicked() {
            action = ImportAction::PickFile;
        }
        if !state.path_label.is_empty() {
            ui.label(
                RichText::new(&state.path_label)
                    .small()
                    .color(Color32::GRAY),
            );
        }
    });

    if state.extracted.is_empty() {
        if !state.path_label.is_empty() {
            ui.label("No functions found in the selected file.");
        }
        return action;
    }

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label("Target module name:");
        ui.text_edit_singleline(&mut state.target_module);
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.small_button("Select all").clicked() {
            state.selected.iter_mut().for_each(|s| *s = true);
        }
        if ui.small_button("Deselect all").clicked() {
            state.selected.iter_mut().for_each(|s| *s = false);
        }
    });
    ui.separator();

    let selected_count = state.selected.iter().filter(|&&s| s).count();

    ScrollArea::vertical().id_salt("import_scroll").max_height(400.0).show(ui, |ui| {
        for (i, func) in state.extracted.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.selected[i], "");
                ui.vertical(|ui| {
                    ui.label(RichText::new(&func.name).strong().monospace());
                    ui.label(
                        RichText::new(&func.signature)
                            .small()
                            .color(Color32::GRAY)
                            .monospace(),
                    );
                    if !func.comment.is_empty() {
                        ui.collapsing("Comment", |ui| {
                            ui.label(
                                RichText::new(&func.comment)
                                    .small()
                                    .monospace()
                                    .color(Color32::LIGHT_GRAY),
                            );
                        });
                    }
                });
            });
            ui.separator();
        }
    });

    ui.add_space(8.0);
    let can_import = selected_count > 0 && !state.target_module.trim().is_empty();
    ui.horizontal(|ui| {
        if ui
            .add_enabled(can_import, egui::Button::new(format!("Import {selected_count} function(s)")))
            .clicked()
        {
            let module = state.target_module.trim().to_string();
            let funcs: Vec<FunctionTemplate> = state
                .extracted
                .iter()
                .zip(state.selected.iter())
                .filter(|(_, sel)| **sel)
                .map(|(f, _)| FunctionTemplate {
                    name: f.name.clone(),
                    module: module.clone(),
                    description: String::new(),
                    signature: f.signature.clone(),
                    header_code: format!("{};", f.signature),
                    impl_code: format!("{}\n{}", f.signature, f.body),
                    requires: Vec::new(),
                    tags: Vec::new(),
                    notes: if f.comment.is_empty() {
                        String::new()
                    } else {
                        format!("Imported comment:\n{}", f.comment)
                    },
                    starred: false,
                })
                .collect();
            action = ImportAction::Import(funcs);
        }
        if ui.button("Cancel").clicked() {
            action = ImportAction::Cancel;
        }
    });

    action
}
