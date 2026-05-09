use egui::{Color32, RichText, Ui};
use std::path::PathBuf;

use newc_core::{function_lib::FunctionLibrary, project_template};

pub enum CreateAction {
    None,
    Create {
        name: String,
        git: bool,
        author: String,
        include_input: bool,
        include_math: bool,
        include_display: bool,
        include_array: bool,
        location: PathBuf,
    },
    Cancel,
}

pub fn show(
    ui: &mut Ui,
    name: &mut String,
    git: &mut bool,
    author: &mut String,
    include_input: &mut bool,
    include_math: &mut bool,
    include_display: &mut bool,
    include_array: &mut bool,
    func_search: &mut String,
    func_selected: &mut Vec<String>,
    lib: &FunctionLibrary,
    _show_func_picker: &mut bool,
    selected_template: &mut Option<usize>,
) -> CreateAction {
    ui.heading("New Project");
    ui.separator();

    // ── Template picker ───────────────────────────────────────────────────────
    let templates = project_template::all_templates();
    ui.label(RichText::new("Start from template (optional):").strong());
    ui.horizontal_wrapped(|ui| {
        let none_sel = selected_template.is_none();
        if ui.selectable_label(none_sel, "Blank").clicked() {
            *selected_template = None;
        }
        for (i, t) in templates.iter().enumerate() {
            let sel = *selected_template == Some(i);
            if ui.selectable_label(sel, t.name).on_hover_text(t.description).clicked() {
                *selected_template = Some(i);
                // Pre-check modules matching template
                let mods = t.modules;
                *include_input   = mods.contains(&"input");
                *include_math    = mods.contains(&"math");
                *include_display = mods.contains(&"display");
                *include_array   = mods.contains(&"array");
            }
        }
    });
    if let Some(idx) = selected_template {
        let t = &templates[*idx];
        ui.label(RichText::new(format!("→ {}", t.description)).small().color(Color32::GRAY));
    }
    ui.separator();

    egui::Grid::new("create_grid").num_columns(2).show(ui, |ui| {
        ui.label("Project name:");
        ui.text_edit_singleline(name);
        ui.end_row();

        ui.label("Author:");
        ui.text_edit_singleline(author);
        ui.end_row();

        ui.label("Init git:");
        ui.checkbox(git, "");
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.label("Default modules:");
    ui.horizontal(|ui| {
        ui.checkbox(include_input, "input");
        ui.checkbox(include_math, "math");
        ui.checkbox(include_display, "display");
        ui.checkbox(include_array, "array");
    });

    ui.add_space(8.0);
    egui::CollapsingHeader::new("Function Library (optional pre-selection)")
        .default_open(true)
        .show(ui, |ui| {
            super::function_picker::show(ui, lib, func_search, func_selected);
        });

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        let can_create = !name.trim().is_empty();
        if ui.add_enabled(can_create, egui::Button::new("Create")).clicked() {
            // Create in home dir by default; user can change later
            let location = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            return CreateAction::Create {
                name: name.trim().to_string(),
                git: *git,
                author: author.clone(),
                include_input: *include_input,
                include_math: *include_math,
                include_display: *include_display,
                include_array: *include_array,
                location,
            };
        }
        if ui.button("Cancel").clicked() {
            return CreateAction::Cancel;
        }
        CreateAction::None
    })
    .inner
}
