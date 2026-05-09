use std::collections::HashMap;

use egui::{Color32, Context, RichText, ScrollArea};
use newc_core::function_lib::FunctionLibrary;
use newc_core::project::Project;

use crate::state::{AppState, View};

pub fn show(ctx: &Context, state: &mut AppState) {
    let project = match &state.view {
        View::UsageTracker(p) => p.clone(),
        _ => return,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.view = View::ProjectDetail(project.clone());
                return;
            }
            ui.heading(format!("Function Usage — {}", project.name));
        });
        ui.separator();

        let usage_map = compute_usage(&project, &state.function_lib);

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut state.usage_search);
            if ui.small_button("✕").clicked() {
                state.usage_search.clear();
            }
        });
        ui.add_space(4.0);

        if usage_map.is_empty() {
            ui.label(RichText::new("No library function usage found.").color(Color32::GRAY));
            return;
        }

        let q = state.usage_search.to_lowercase();

        ScrollArea::vertical().show(ui, |ui| {
            // Group by module
            let mut by_module: HashMap<String, Vec<(&String, &Vec<String>)>> = HashMap::new();
            for (fname, files) in &usage_map {
                if !q.is_empty() && !fname.to_lowercase().contains(&q) {
                    continue;
                }
                // Find which module this function belongs to
                let module = state.function_lib.all().iter()
                    .find(|f| &f.name == fname)
                    .map(|f| f.module.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                by_module.entry(module).or_default().push((fname, files));
            }

            let mut modules: Vec<String> = by_module.keys().cloned().collect();
            modules.sort();

            if modules.is_empty() {
                ui.label(RichText::new("No matches.").color(Color32::GRAY));
                return;
            }

            for module in &modules {
                let entries = &by_module[module];
                ui.collapsing(
                    RichText::new(format!("{} ({})", module, entries.len())).strong(),
                    |ui| {
                        egui::Grid::new(format!("usage_{module}"))
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                for (fname, files) in entries.iter() {
                                    ui.label(RichText::new(*fname).monospace());
                                    ui.label(files.join(", "));
                                    ui.end_row();
                                }
                            });
                    },
                );
            }
        });
    });
}

fn compute_usage(project: &Project, lib: &FunctionLibrary) -> HashMap<String, Vec<String>> {
    let src_dir = project.root.join("src");
    let Ok(entries) = std::fs::read_dir(&src_dir) else {
        return HashMap::new();
    };

    let func_names: Vec<&str> = lib.all().iter().map(|f| f.name.as_str()).collect();
    let mut usage: HashMap<String, Vec<String>> = HashMap::new();

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("c"))
        .map(|e| e.path())
        .collect();
    paths.sort();

    for path in &paths {
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        for fname in &func_names {
            // Simple substring check — function call pattern
            if content.contains(&format!("{fname}(")) {
                usage.entry(fname.to_string()).or_default().push(file_name.clone());
            }
        }
    }

    usage
}
