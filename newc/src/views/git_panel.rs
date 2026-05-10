use egui::{Color32, Context, Frame, RichText, ScrollArea};
use newc_core::git;

use crate::state::{AppState, View};

pub fn show(ctx: &Context, state: &mut AppState) {
    let project = match &state.view {
        View::GitPanel(p) => p.clone(),
        _ => return,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        // ── Header ────────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.view = View::ProjectDetail(project.clone());
                return;
            }
            ui.heading(RichText::new(format!("◆ Git — {}", project.name)).color(Color32::from_rgb(169, 220, 118)));

            // Branch management
            ui.separator();
            let branches = git::branches(&project.root);
            let cur_branch = git::current_branch(&project.root);
            if !branches.is_empty() {
                egui::ComboBox::from_id_salt("branch_combo")
                    .selected_text(format!("⎇ {cur_branch}"))
                    .show_ui(ui, |ui| {
                        for branch in &branches {
                            if ui.selectable_label(branch == &cur_branch, branch).clicked() && branch != &cur_branch {
                                match git::switch_branch(&project.root, branch) {
                                    Ok(_) => state.set_status(format!("Switched to {branch}")),
                                    Err(e) => state.set_error(e.to_string()),
                                }
                            }
                        }
                    });
            }

            // New branch
            ui.add(
                egui::TextEdit::singleline(&mut state.git_new_branch)
                    .hint_text("New branch…")
                    .desired_width(120.0),
            );
            if ui.add_enabled(!state.git_new_branch.trim().is_empty(), egui::Button::new("Create Branch")).clicked() {
                let name = state.git_new_branch.trim().to_string();
                match git::new_branch(&project.root, &name) {
                    Ok(_) => {
                        state.set_status(format!("Created + switched to {name}"));
                        state.git_new_branch.clear();
                    }
                    Err(e) => state.set_error(e.to_string()),
                }
            }
        });
        ui.separator();

        let is_repo = git::is_repo(&project.root);

        if !is_repo {
            ui.label(RichText::new("No git repository found.").color(ui.visuals().warn_fg_color));
            ui.add_space(8.0);
            if ui.button("Init repository").clicked() {
                match git::init(&project.root) {
                    Ok(_) => state.set_status("Repository initialised"),
                    Err(e) => state.set_error(format!("git init failed: {e}")),
                }
            }
            return;
        }

        // ── Status summary ────────────────────────────────────────────────────
        if let Some(status) = git::status(&project.root) {
            Frame::new()
                .inner_margin(egui::Margin::same(8))
                .corner_radius(egui::CornerRadius::same(4))
                .fill(ui.visuals().faint_bg_color)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Branch:").strong());
                        ui.label(RichText::new(&status.branch).monospace().color(Color32::from_rgb(80, 200, 120)));
                        ui.separator();
                        ui.label(format!("staged: {}", status.staged));
                        ui.separator();
                        ui.label(format!("unstaged: {}", status.unstaged));
                        ui.separator();
                        ui.label(format!("untracked: {}", status.untracked));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Pull").clicked() {
                                match git::pull(&project.root) {
                                    Ok(out) => {
                                        state.set_status("Pull complete");
                                        state.build_lines.push(crate::build_runner::BuildLine { text: out, kind: crate::build_runner::LineKind::Info });
                                        state.build_panel_open_hint = true;
                                    }
                                    Err(e) => state.set_error(e.to_string()),
                                }
                            }
                            if ui.button("Push").clicked() {
                                match git::push(&project.root) {
                                    Ok(out) => {
                                        state.set_status("Push complete");
                                        state.build_lines.push(crate::build_runner::BuildLine { text: out, kind: crate::build_runner::LineKind::Info });
                                        state.build_panel_open_hint = true;
                                    }
                                    Err(e) => state.set_error(e.to_string()),
                                }
                            }
                        });
                    });
                });
        }

        ui.add_space(6.0);

        // ── Per-file staging ──────────────────────────────────────────────────
        let changed = git::changed_files(&project.root);
        if !changed.is_empty() {
            ui.label(RichText::new("Changed Files").strong());
            egui::Grid::new("changed_files_grid")
                .num_columns(4)
                .striped(true)
                .spacing([8.0, 3.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Staged").strong().small());
                    ui.label(RichText::new("File").strong().small());
                    ui.label(RichText::new("Status").strong().small());
                    ui.label("");
                    ui.end_row();

                    for file in &changed {
                        let mut is_staged = file.staged;
                        if ui.checkbox(&mut is_staged, "").changed() {
                            let result = if is_staged {
                                git::stage_file(&project.root, &file.path)
                            } else {
                                git::unstage_file(&project.root, &file.path)
                            };
                            if let Err(e) = result {
                                state.set_error(e.to_string());
                            }
                        }
                        ui.label(RichText::new(&file.path).monospace().small());
                        let status = match (file.staged, file.unstaged, file.untracked) {
                            (_, _, true) => RichText::new("untracked").small().color(Color32::GRAY),
                            (true, true, _) => RichText::new("staged+modified").small().color(Color32::from_rgb(255, 200, 60)),
                            (true, _, _) => RichText::new("staged").small().color(Color32::from_rgb(100, 220, 100)),
                            (_, true, _) => RichText::new("modified").small().color(Color32::from_rgb(255, 140, 0)),
                            _ => RichText::new("?").small(),
                        };
                        ui.label(status);
                        ui.end_row();
                    }
                });
            ui.add_space(4.0);
        }

        // ── Commit ────────────────────────────────────────────────────────────
        let can_commit = !state.git_commit_msg.trim().is_empty();
        ui.label(RichText::new("Commit staged files").strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.git_commit_msg)
                    .hint_text("Commit message…")
                    .desired_width(300.0),
            );
            if ui.add_enabled(can_commit, egui::Button::new("Commit")).clicked() {
                let msg = state.git_commit_msg.trim().to_string();
                match git::commit(&project.root, &msg) {
                    Ok(_) => {
                        state.git_commit_msg.clear();
                        state.set_status("Committed");
                    }
                    Err(e) => state.set_error(format!("Commit failed: {e}")),
                }
            }
            if ui.button("Stage All").clicked() {
                if let Err(e) = git::stage_all(&project.root) {
                    state.set_error(e.to_string());
                }
            }
        });

        ui.add_space(8.0);

        // ── Diff view ─────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(RichText::new("Diff").strong());
            ui.toggle_value(&mut state.git_show_diff, "Show Diff");
            if state.git_show_diff {
                ui.toggle_value(&mut state.git_diff_staged, "Staged");
            }
        });

        if state.git_show_diff {
            let diff_text = if state.git_diff_staged {
                git::diff_staged(&project.root)
            } else {
                git::diff(&project.root)
            };
            if diff_text.is_empty() {
                ui.label(RichText::new("No changes.").color(Color32::GRAY));
            } else {
                Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for line in diff_text.lines() {
                                let color = if line.starts_with('+') && !line.starts_with("+++") {
                                    Color32::from_rgb(100, 200, 100)
                                } else if line.starts_with('-') && !line.starts_with("---") {
                                    Color32::from_rgb(200, 100, 100)
                                } else if line.starts_with("@@") {
                                    Color32::from_rgb(100, 150, 220)
                                } else {
                                    Color32::LIGHT_GRAY
                                };
                                ui.label(RichText::new(line).monospace().size(11.0).color(color));
                            }
                        });
                    });
            }
            ui.add_space(6.0);
        }

        // ── Commit log ────────────────────────────────────────────────────────
        let commits = git::log(&project.root, 20);
        if !commits.is_empty() {
            ui.label(RichText::new(format!("Log ({} shown)", commits.len())).strong());
            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                egui::Grid::new("git_log_grid")
                    .num_columns(3)
                    .striped(true)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        for commit in &commits {
                            ui.label(
                                RichText::new(&commit.hash[..7.min(commit.hash.len())])
                                    .monospace()
                                    .color(Color32::from_rgb(180, 140, 80)),
                            );
                            ui.label(RichText::new(&commit.date).small().color(Color32::GRAY));
                            ui.label(&commit.message);
                            ui.end_row();
                        }
                    });
            });
        }
    });
}
