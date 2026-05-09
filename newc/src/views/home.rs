use egui::Ui;
use std::path::PathBuf;

pub struct HomeAction {
    pub open_project: Option<PathBuf>,
    pub remove_project: Option<usize>,
    pub go_create: bool,
    pub browse_for_project: bool,
}

pub fn show(ui: &mut Ui, known_projects: &[PathBuf]) -> HomeAction {
    let mut action = HomeAction {
        open_project: None,
        remove_project: None,
        go_create: false,
        browse_for_project: false,
    };

    ui.heading("Projects");
    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("New Project").clicked() {
            action.go_create = true;
        }
        if ui.button("Open Folder...").clicked() {
            action.browse_for_project = true;
        }
    });
    ui.add_space(8.0);

    if known_projects.is_empty() {
        ui.label("No projects yet. Create one or open an existing folder.");
        return action;
    }

    let mut to_remove: Option<usize> = None;
    egui::Grid::new("projects_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for (i, path) in known_projects.iter().enumerate() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                let exists = path.exists();
                let label = if exists {
                    name
                } else {
                    format!("{name} (missing)")
                };

                if ui.selectable_label(false, &label).clicked() && exists {
                    action.open_project = Some(path.clone());
                }
                ui.label(
                    egui::RichText::new(path.display().to_string())
                        .small()
                        .color(egui::Color32::GRAY),
                );
                if ui.small_button("✕").clicked() {
                    to_remove = Some(i);
                }
                ui.end_row();
            }
        });

    if let Some(i) = to_remove {
        action.remove_project = Some(i);
    }

    action
}
