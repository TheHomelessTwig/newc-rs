use egui::{Color32, Context, RichText, ScrollArea};
use newc_core::build_history;
use newc_core::project::Project;

use crate::state::{AppState, View};

pub fn show(ctx: &Context, state: &mut AppState) {
    let project = match &state.view {
        View::BuildHistory(p) => p.clone(),
        _ => return,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.view = View::ProjectDetail(project.clone());
                return;
            }
            ui.heading(format!("Build History — {}", project.name));
        });
        ui.separator();

        let records = build_history::load(&project.root);

        if records.is_empty() {
            ui.add_space(20.0);
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No build history yet. Run a build first.").color(Color32::GRAY));
            });
            return;
        }

        ui.label(RichText::new(format!("{} build records (last 100 kept)", records.len())).small().color(Color32::GRAY));
        ui.add_space(4.0);

        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("build_history_grid")
                .num_columns(4)
                .striped(true)
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Timestamp").strong());
                    ui.label(RichText::new("Target").strong());
                    ui.label(RichText::new("Result").strong());
                    ui.label(RichText::new("Duration").strong());
                    ui.end_row();

                    for rec in records.iter().rev() {
                        ui.label(RichText::new(&rec.timestamp).monospace().small());
                        ui.label(RichText::new(&rec.target).monospace());
                        if rec.succeeded() {
                            ui.label(RichText::new("✓ OK").color(Color32::from_rgb(100, 220, 100)));
                        } else {
                            let code = rec.exit_code.map(|c| format!("✗ exit {c}")).unwrap_or_else(|| "✗ killed".to_string());
                            ui.label(RichText::new(code).color(Color32::from_rgb(255, 80, 80)));
                        }
                        ui.label(rec.duration_str());
                        ui.end_row();
                    }
                });
        });
    });
}
