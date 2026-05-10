use egui::{Color32, RichText, Ui};
use std::path::PathBuf;

use newc_core::{function_lib::FunctionLibrary, module::is_valid_c_ident, project_template, scaffold::detect_author};
use rfd::FileDialog;

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
        include_strings: bool,
        include_linked_list: bool,
        include_files: bool,
        include_test_utils: bool,
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
    include_strings: &mut bool,
    include_linked_list: &mut bool,
    include_files: &mut bool,
    include_test_utils: &mut bool,
    func_search: &mut String,
    func_selected: &mut Vec<String>,
    lib: &FunctionLibrary,
    _show_func_picker: &mut bool,
    selected_template: &mut Option<usize>,
    location: &mut String,
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

    let name_valid = name.trim().is_empty() || is_valid_c_ident(name.trim());
    let loc_path = PathBuf::from(location.as_str());
    let loc_valid = location.trim().is_empty() || loc_path.exists();

    egui::Grid::new("create_grid").num_columns(2).show(ui, |ui| {
        ui.label("Project name:");
        ui.vertical(|ui| {
            ui.text_edit_singleline(name);
            if !name_valid {
                ui.label(RichText::new("Must be a valid C identifier (letters, digits, underscores; no spaces)").small().color(Color32::RED));
            }
        });
        ui.end_row();

        ui.label("Author:");
        ui.text_edit_singleline(author);
        ui.end_row();

        ui.label("Location:");
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(location).desired_width(260.0));
                if ui.small_button("Browse…").clicked() {
                    if let Some(picked) = FileDialog::new().pick_folder() {
                        *location = picked.to_string_lossy().into_owned();
                    }
                }
            });
            if !loc_valid {
                ui.label(RichText::new("Path does not exist").small().color(Color32::RED));
            }
        });
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
        ui.checkbox(include_strings, "strings");
        ui.checkbox(include_linked_list, "linked_list");
        ui.checkbox(include_files, "files");
        ui.checkbox(include_test_utils, "test_utils");
    });

    ui.add_space(8.0);
    egui::CollapsingHeader::new("Function Library (optional pre-selection)")
        .default_open(true)
        .show(ui, |ui| {
            super::function_picker::show(ui, lib, func_search, func_selected);
        });

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        let can_create = !name.trim().is_empty() && name_valid && loc_valid;
        if ui.add_enabled(can_create, egui::Button::new("Create")).clicked() {
            let location = if location.trim().is_empty() {
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
            } else {
                PathBuf::from(location.as_str())
            };
            // Fall back to git-detected author if field left blank
            let resolved_author = if author.trim().is_empty() {
                detect_author()
            } else {
                author.trim().to_string()
            };
            return CreateAction::Create {
                name: name.trim().to_string(),
                git: *git,
                author: resolved_author,
                include_input: *include_input,
                include_math: *include_math,
                include_display: *include_display,
                include_array: *include_array,
                include_strings: *include_strings,
                include_linked_list: *include_linked_list,
                include_files: *include_files,
                include_test_utils: *include_test_utils,
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
