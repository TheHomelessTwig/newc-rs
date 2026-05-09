use egui::Ui;
use newc_core::function_lib::FunctionLibrary;
use newc_core::project::Project;

pub enum AddModuleAction {
    None,
    Create { name: String, selected_funcs: Vec<String> },
    Cancel,
}

pub fn show(
    ui: &mut Ui,
    project: &Project,
    name: &mut String,
    func_search: &mut String,
    func_selected: &mut Vec<String>,
    lib: &FunctionLibrary,
) -> AddModuleAction {
    ui.horizontal(|ui| {
        if ui.button("← Back").clicked() {
            return AddModuleAction::Cancel;
        }
        ui.heading(format!("Add Module — {}", project.name));
        AddModuleAction::None
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Module name:");
        ui.text_edit_singleline(name);
    });

    ui.add_space(8.0);
    ui.label("Pick functions to pre-populate (optional):");

    super::function_picker::show(ui, lib, func_search, func_selected);

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let can_create = !name.trim().is_empty();
        if ui.add_enabled(can_create, egui::Button::new("Create Module")).clicked() {
            return AddModuleAction::Create {
                name: name.trim().to_string(),
                selected_funcs: func_selected.clone(),
            };
        }
        if ui.button("Cancel").clicked() {
            return AddModuleAction::Cancel;
        }
        AddModuleAction::None
    })
    .inner
}
