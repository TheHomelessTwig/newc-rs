use egui::{Color32, Context, Key, RichText, ScrollArea};
use newc_core::grep;

use crate::state::{AppState, View};

pub fn show(ctx: &Context, state: &mut AppState) {
    let project = match &state.view {
        View::ProjectSearch(p) => p.clone(),
        _ => return,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.view = View::ProjectDetail(project.clone());
                return;
            }
            ui.heading(format!("Search — {}", project.name));
        });
        ui.separator();

        ui.horizontal(|ui| {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text("Search across all .c and .h files…")
                    .desired_width(360.0),
            );
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if ui.button("Search").clicked() || enter {
                state.search_results = grep::search(&project.root, &state.search_query);
            }
            if ui.small_button("✕").clicked() {
                state.search_query.clear();
                state.search_results.clear();
            }
        });
        ui.add_space(4.0);

        if state.search_query.is_empty() {
            ui.label(RichText::new("Enter a query and press Enter or click Search.").color(Color32::GRAY));
            return;
        }

        if state.search_results.is_empty() && !state.search_query.is_empty() {
            ui.label(RichText::new("No results.").color(Color32::GRAY));
            return;
        }

        ui.label(
            RichText::new(format!("{} result(s)", state.search_results.len()))
                .small()
                .color(Color32::GRAY),
        );
        ui.add_space(4.0);

        let results = state.search_results.clone();
        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("search_results_grid")
                .num_columns(3)
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("File").strong());
                    ui.label(RichText::new("Line").strong());
                    ui.label(RichText::new("Text").strong());
                    ui.end_row();

                    for result in &results {
                        let file_label = egui::Label::new(
                            RichText::new(&result.file)
                                .monospace()
                                .color(Color32::from_rgb(100, 180, 255)),
                        )
                        .sense(egui::Sense::click());
                        if ui.add(file_label).clicked() {
                            if let Some(module_name) = &result.module {
                                state.module_detail_state = Default::default();
                                state.view = View::ModuleDetail {
                                    project: project.clone(),
                                    module_name: module_name.clone(),
                                };
                            }
                        }

                        ui.label(
                            RichText::new(result.line_no.to_string())
                                .monospace()
                                .small()
                                .color(Color32::GRAY),
                        );

                        // Highlight the query within the line
                        let line = &result.text;
                        if let Some(pos) = line.to_lowercase().find(&state.search_query.to_lowercase()) {
                            let before = &line[..pos];
                            let matched = &line[pos..pos + state.search_query.len()];
                            let after = &line[pos + state.search_query.len()..];
                            let mut layoutjob = egui::text::LayoutJob::default();
                            let font = egui::FontId::monospace(12.0);
                            let gray = egui::TextFormat { font_id: font.clone(), color: Color32::LIGHT_GRAY, ..Default::default() };
                            let highlight = egui::TextFormat { font_id: font.clone(), color: Color32::from_rgb(255, 220, 50), ..Default::default() };
                            layoutjob.append(before, 0.0, gray.clone());
                            layoutjob.append(matched, 0.0, highlight);
                            layoutjob.append(after, 0.0, gray);
                            ui.label(layoutjob);
                        } else {
                            ui.label(RichText::new(line).monospace().small());
                        }
                        ui.end_row();
                    }
                });
        });
    });
}
