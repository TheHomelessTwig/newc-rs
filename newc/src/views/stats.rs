use egui::{Color32, RichText, Ui};
use newc_core::{project::Project, stats::ProjectStats};

pub fn show(ui: &mut Ui, project: &Project, stats: &ProjectStats) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("◈").color(Color32::from_rgb(171, 157, 242)).heading());
        ui.heading(RichText::new(format!("Stats — {}", project.name)).color(Color32::from_rgb(252, 252, 250)));
    });
    ui.separator();

    // Summary row
    egui::Grid::new("stats_summary").num_columns(2).show(ui, |ui| {
        ui.label(RichText::new("◆ Modules").strong().color(Color32::from_rgb(120, 220, 232)));
        ui.label(RichText::new(stats.module_stats.len().to_string()).color(Color32::from_rgb(171, 157, 242)));
        ui.end_row();

        ui.label(RichText::new("◆ Total functions").strong().color(Color32::from_rgb(120, 220, 232)));
        ui.label(RichText::new(stats.total_functions.to_string()).color(Color32::from_rgb(171, 157, 242)));
        ui.end_row();

        ui.label(RichText::new("◆ Lines of code (LOC)").strong().color(Color32::from_rgb(120, 220, 232)));
        ui.label(RichText::new(stats.total_loc.to_string()).color(Color32::from_rgb(171, 157, 242)));
        ui.end_row();

        ui.label(RichText::new("◆ Total source lines").strong().color(Color32::from_rgb(120, 220, 232)));
        ui.label(RichText::new(stats.total_source_lines.to_string()).color(Color32::from_rgb(171, 157, 242)));
        ui.end_row();
    });

    ui.add_space(12.0);
    ui.label(RichText::new("◈ Per-module breakdown").strong().color(Color32::from_rgb(120, 220, 232)));
    ui.separator();

    if stats.module_stats.is_empty() {
        ui.label(RichText::new("No modules found.").color(Color32::GRAY));
        return;
    }

    // Simple bar chart — widths proportional to LOC
    let max_loc = stats.module_stats.iter().map(|m| m.loc).max().unwrap_or(1).max(1);

    egui::Grid::new("stats_modules")
        .num_columns(4)
        .striped(true)
        .show(ui, |ui| {
            ui.label(RichText::new("Module").strong().color(Color32::from_rgb(171, 157, 242)));
            ui.label(RichText::new("fns").strong().color(Color32::from_rgb(171, 157, 242)));
            ui.label(RichText::new("LOC").strong().color(Color32::from_rgb(171, 157, 242)));
            ui.label(RichText::new("").strong());
            ui.end_row();

            for m in &stats.module_stats {
                ui.label(RichText::new(format!("◆ {}", m.name)).color(Color32::from_rgb(169, 220, 118)));
                ui.label(RichText::new(m.functions.to_string()).color(Color32::from_rgb(255, 216, 102)));
                ui.label(RichText::new(m.loc.to_string()).color(Color32::from_rgb(255, 216, 102)));

                // Bar proportional to LOC
                let bar_width = (m.loc as f32 / max_loc as f32) * 150.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(160.0, 14.0),
                    egui::Sense::hover(),
                );
                let bar_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(bar_width, rect.height()),
                );
                let hue = (m.loc as f32 / max_loc as f32) * 0.4; // green → yellow
                let color = egui::ecolor::Hsva::new(0.33 - hue, 0.7, 0.8, 1.0);
                ui.painter().rect_filled(bar_rect, 2.0, Color32::from(color));
                ui.painter().rect_stroke(
                    rect,
                    2.0,
                    egui::Stroke::new(0.5, Color32::DARK_GRAY),
                    egui::StrokeKind::Outside,
                );
                ui.end_row();
            }
        });

    ui.add_space(8.0);
    ui.label(
        RichText::new("LOC = non-blank, non-comment lines in src/*.c files.")
            .small()
            .color(Color32::GRAY),
    );
}
