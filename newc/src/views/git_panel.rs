use egui::{Context, Frame, RichText, ScrollArea};
use newc_core::git;

use crate::state::{AppState, View};

pub fn show(ctx: &Context, state: &mut AppState) {
    let project = match &state.view {
        View::GitPanel(p) => p.clone(),
        _ => return,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        // Header
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.view = View::ProjectDetail(project.clone());
                return;
            }
            ui.heading(format!("Git — {}", project.name));
        });
        ui.separator();
        ui.add_space(4.0);

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

        // Status block
        if let Some(status) = git::status(&project.root) {
            Frame::new()
                .inner_margin(egui::Margin::same(8))
                .corner_radius(egui::CornerRadius::same(4))
                .fill(ui.visuals().faint_bg_color)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Branch:").strong());
                        ui.label(RichText::new(&status.branch).monospace().color(
                            egui::Color32::from_rgb(80, 200, 120),
                        ));
                        ui.separator();
                        ui.label(RichText::new(format!("staged: {}", status.staged)).strong());
                        ui.separator();
                        ui.label(format!("unstaged: {}", status.unstaged));
                        ui.separator();
                        ui.label(format!("untracked: {}", status.untracked));
                    });
                });
        }

        ui.add_space(8.0);

        // Commit panel
        ui.label(RichText::new("Commit").strong());
        let can_commit = !state.git_commit_msg.trim().is_empty();
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.git_commit_msg)
                    .hint_text("Commit message…")
                    .desired_width(300.0),
            );
            if ui.add_enabled(can_commit, egui::Button::new("Stage all + Commit")).clicked() {
                let msg = state.git_commit_msg.trim().to_string();
                match git::stage_all(&project.root).and_then(|_| git::commit(&project.root, &msg)) {
                    Ok(_) => {
                        state.git_commit_msg.clear();
                        state.set_status("Committed");
                    }
                    Err(e) => state.set_error(format!("Commit failed: {e}")),
                }
            }
        });

        ui.add_space(12.0);

        // Log
        let commits = git::log(&project.root, 20);
        if commits.is_empty() {
            ui.label("No commits yet.");
        } else {
            ui.label(RichText::new(format!("Log ({} shown)", commits.len())).strong());
            ui.add_space(4.0);
            ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                egui::Grid::new("git_log_grid")
                    .num_columns(3)
                    .striped(true)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        for commit in &commits {
                            ui.label(
                                RichText::new(&commit.hash[..7.min(commit.hash.len())])
                                    .monospace()
                                    .color(egui::Color32::from_rgb(180, 140, 80)),
                            );
                            ui.label(&commit.date);
                            ui.label(&commit.message);
                            ui.end_row();
                        }
                    });
            });
        }

        ui.add_space(8.0);
    });
}
