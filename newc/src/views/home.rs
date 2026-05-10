use egui::{Color32, RichText, Ui};
use std::path::PathBuf;

use newc_core::config::AppConfig;

pub struct HomeAction {
    pub open_project: Option<PathBuf>,
    pub remove_project: Option<usize>,
    pub go_create: bool,
    pub browse_for_project: bool,
    pub archive_project: Option<PathBuf>,
    pub move_to_workspace: Option<(PathBuf, String)>,
    pub new_workspace: Option<String>,
}

pub fn show(
    ui: &mut Ui,
    known_projects: &[PathBuf],
    is_first_run: bool,
    config: &AppConfig,
    active_workspace: &mut Option<String>,
    show_archived: &mut bool,
    workspace_input: &mut String,
    show_new_workspace: &mut bool,
) -> HomeAction {
    let mut action = HomeAction {
        open_project: None,
        remove_project: None,
        go_create: false,
        browse_for_project: false,
        archive_project: None,
        move_to_workspace: None,
        new_workspace: None,
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new("⌂").color(Color32::from_rgb(255, 216, 102)).heading());
        ui.heading(RichText::new("Projects").color(Color32::from_rgb(252, 252, 250)));
    });
    ui.separator();

    // First-run onboarding
    if is_first_run && known_projects.is_empty() {
        egui::Frame::new()
            .inner_margin(egui::Margin::same(12))
            .corner_radius(egui::CornerRadius::same(6))
            .fill(ui.visuals().faint_bg_color)
            .show(ui, |ui| {
                ui.label(RichText::new("Welcome to newc!").heading());
                ui.add_space(6.0);
                ui.label("newc helps you scaffold, build and manage C projects with reusable function libraries.");
                ui.add_space(8.0);
                ui.label(RichText::new("Getting started:").strong());
                ui.label("1. Create your first project with \"New Project\"");
                ui.label("2. Add modules and import functions from the Function Library");
                ui.label("3. Use Compose main() to visually build your main.c");
                ui.label("4. Browse the C Reference for stdlib documentation");
                ui.add_space(8.0);
                if ui.button("Create first project").clicked() {
                    action.go_create = true;
                }
            });
        ui.add_space(12.0);
    }

    // ── Workspace tabs ────────────────────────────────────────────────────────
    ui.horizontal_wrapped(|ui| {
        let all_sel = active_workspace.is_none() && !*show_archived;
        if ui.selectable_label(all_sel, "All").clicked() {
            *active_workspace = None;
            *show_archived = false;
        }
        for ws in &config.workspaces {
            let is_sel = active_workspace.as_deref() == Some(&ws.name);
            if ui.selectable_label(is_sel, &ws.name).clicked() {
                *active_workspace = Some(ws.name.clone());
                *show_archived = false;
            }
        }
        let archived_sel = *show_archived;
        if ui.selectable_label(archived_sel, "Archived").clicked() {
            *show_archived = !*show_archived;
            *active_workspace = None;
        }
        if ui.small_button("+").on_hover_text("New workspace").clicked() {
            *show_new_workspace = true;
        }
    });

    // New workspace input
    if *show_new_workspace {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(workspace_input);
            let valid = !workspace_input.trim().is_empty();
            if ui.add_enabled(valid, egui::Button::new("Create")).clicked() {
                action.new_workspace = Some(workspace_input.trim().to_string());
                workspace_input.clear();
                *show_new_workspace = false;
            }
            if ui.button("Cancel").clicked() {
                *show_new_workspace = false;
                workspace_input.clear();
            }
        });
    }

    ui.add_space(4.0);

    // Toolbar
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("+ New Project").fill(Color32::from_rgb(40, 90, 50))).clicked() {
            action.go_create = true;
        }
        if ui.button("Open Folder...").clicked() {
            action.browse_for_project = true;
        }
    });
    ui.add_space(8.0);

    if known_projects.is_empty() && !is_first_run {
        ui.label("No projects yet. Create one or open an existing folder.");
        return action;
    } else if known_projects.is_empty() {
        return action;
    }

    // Filter projects for current workspace/archived view
    let archived_paths: Vec<PathBuf> = config
        .workspaces
        .iter()
        .find(|w| w.name == "__archived__")
        .map(|w| w.paths.clone())
        .unwrap_or_default();

    let workspace_paths: Option<&[PathBuf]> = active_workspace.as_ref().and_then(|ws_name| {
        config.workspaces.iter().find(|w| &w.name == ws_name).map(|w| w.paths.as_slice())
    });

    let visible_projects: Vec<(usize, &PathBuf)> = known_projects
        .iter()
        .enumerate()
        .filter(|(_, path)| {
            let is_archived = archived_paths.contains(path);
            if *show_archived {
                return is_archived;
            }
            if is_archived {
                return false;
            }
            match &workspace_paths {
                Some(ws_paths) => ws_paths.contains(path),
                None => true,
            }
        })
        .collect();

    if visible_projects.is_empty() {
        ui.label(RichText::new("No projects in this workspace.").color(Color32::GRAY));
        return action;
    }

    let mut to_remove: Option<usize> = None;
    egui::Grid::new("projects_grid")
        .num_columns(4)
        .striped(true)
        .show(ui, |ui| {
            for (i, path) in &visible_projects {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                let exists = path.exists();
                let (icon, name_color) = if exists {
                    ("⌂", Color32::from_rgb(169, 220, 118))
                } else {
                    ("⚠", Color32::from_rgb(255, 97, 136))
                };
                let label = if exists {
                    RichText::new(format!("{icon} {name}")).color(name_color).strong()
                } else {
                    RichText::new(format!("{icon} {name} (missing)")).color(name_color)
                };

                if ui.selectable_label(false, label).clicked() && exists {
                    action.open_project = Some((*path).clone());
                }
                ui.label(
                    RichText::new(path.display().to_string())
                        .small()
                        .color(Color32::GRAY),
                );

                // Workspace move menu
                let popup_id = egui::Id::new(("proj_menu", i));
                let response = ui.small_button("...").on_hover_text("Project options");
                if response.clicked() {
                    ui.memory_mut(|m| m.toggle_popup(popup_id));
                }
                egui::popup::popup_below_widget(ui, popup_id, &response, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                    ui.set_min_width(140.0);
                    if !*show_archived {
                        if ui.button("Archive").clicked() {
                            action.archive_project = Some((*path).clone());
                        }
                        for ws in &config.workspaces {
                            if ws.name == "__archived__" { continue; }
                            let in_ws = ws.paths.contains(path);
                            let label = if in_ws {
                                format!("✓ {}", ws.name)
                            } else {
                                format!("Move to {}", ws.name)
                            };
                            if ui.button(label).clicked() && !in_ws {
                                action.move_to_workspace = Some(((*path).clone(), ws.name.clone()));
                            }
                        }
                    } else if ui.button("Unarchive").clicked() {
                        // Move out of archived
                        action.move_to_workspace = Some(((*path).clone(), "__unarchive__".to_string()));
                    }
                });

                if ui.small_button("X").clicked() {
                    to_remove = Some(*i);
                }
                ui.end_row();
            }
        });

    if let Some(i) = to_remove {
        action.remove_project = Some(i);
    }

    action
}
