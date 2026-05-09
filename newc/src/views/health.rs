use egui::{Color32, Context, RichText, ScrollArea};
use newc_core::{analysis, build_history, grep, project::Project, sync::extract_function_implementations};

use crate::state::{AppState, View};
use crate::views::module_detail::compute_missing_includes_for_health;

pub fn show(ctx: &Context, state: &mut AppState) {
    let project = match &state.view {
        View::HealthDashboard(p) => p.clone(),
        _ => return,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.view = View::ProjectDetail(project.clone());
                return;
            }
            ui.heading(format!("Health Dashboard — {}", project.name));
            if ui.button("Refresh").clicked() {
                state.health_computed = false;
            }
        });
        ui.separator();

        if !state.health_computed {
            compute_health(state, &project);
        }

        let snap = state.health_snapshot.clone();

        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                health_card(ui, "Last Build", &snap.last_build_text, snap.last_build_ok);
                health_card(ui, "Dead Code", &snap.dead_code_text, snap.dead_code_count == 0);
                health_card(ui, "Missing Includes", &snap.missing_includes_text, snap.missing_includes_count == 0);
                health_card(ui, "TODO / FIXME", &snap.todos_text, snap.todos_count == 0);
                health_card(ui, "Lint Warnings", &snap.lint_text, snap.lint_count == 0);
            });

            ui.add_space(12.0);

            // TODO list
            if !snap.todos.is_empty() {
                ui.label(RichText::new("TODOs & FIXMEs").strong());
                ui.separator();
                for (file, line_no, text) in &snap.todos {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{file}:{line_no}")).monospace().small().color(Color32::GRAY));
                        ui.label(RichText::new(text).small().color(Color32::from_rgb(255, 210, 60)));
                    });
                }
                ui.add_space(8.0);
            }

            // Dead code list
            if !snap.dead_code_funcs.is_empty() {
                ui.label(RichText::new("Unreachable Functions").strong());
                ui.separator();
                for fname in &snap.dead_code_funcs {
                    ui.label(RichText::new(format!("  {fname}")).monospace().small().color(Color32::from_rgb(255, 140, 0)));
                }
                ui.add_space(8.0);
            }

            // Lint details
            if !snap.lint_warnings.is_empty() {
                ui.label(RichText::new("Lint Warnings").strong());
                ui.separator();
                for (file, code, msg) in &snap.lint_warnings {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("[{code}]")).monospace().small().color(Color32::GRAY));
                        ui.label(RichText::new(format!("{file}: {msg}")).small().color(Color32::from_rgb(255, 200, 80)));
                    });
                }
            }
        });
    });
}

fn health_card(ui: &mut egui::Ui, title: &str, text: &str, ok: bool) {
    let color = if ok { Color32::from_rgb(40, 100, 40) } else { Color32::from_rgb(100, 40, 40) };
    let text_color = if ok { Color32::from_rgb(100, 220, 100) } else { Color32::from_rgb(255, 100, 100) };
    egui::Frame::new()
        .fill(color)
        .inner_margin(egui::Margin::same(10))
        .corner_radius(egui::CornerRadius::same(6))
        .show(ui, |ui| {
            ui.set_min_width(120.0);
            ui.label(RichText::new(title).strong().small());
            ui.label(RichText::new(text).heading().color(text_color));
        });
}

fn compute_health(state: &mut AppState, project: &Project) {
    let mut snap = HealthSnapshot::default();

    // Last build
    let records = build_history::load(&project.root);
    if let Some(last) = records.last() {
        snap.last_build_ok = last.succeeded();
        snap.last_build_text = if last.succeeded() {
            format!("✓ {} ({})", last.target, last.duration_str())
        } else {
            format!("✗ {} ({})", last.target, last.duration_str())
        };
    } else {
        snap.last_build_ok = true;
        snap.last_build_text = "No builds yet".to_string();
    }

    // Dead code
    if let Ok(unreachable) = analysis::check(&project.root) {
        snap.dead_code_count = unreachable.len();
        snap.dead_code_funcs = unreachable.iter().map(|f| f.name.clone()).collect();
        snap.dead_code_text = if unreachable.is_empty() {
            "Clean".to_string()
        } else {
            format!("{} functions", unreachable.len())
        };
    } else {
        snap.dead_code_text = "N/A".to_string();
        snap.last_build_ok = true;
    }

    // Missing includes
    let src_dir = project.root.join("src");
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        let paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("c"))
            .map(|e| e.path())
            .collect();
        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                let warnings = compute_missing_includes_for_health(&content, &state.function_lib);
                snap.missing_includes_count += warnings.len();
            }
        }
    }
    snap.missing_includes_text = if snap.missing_includes_count == 0 {
        "Clean".to_string()
    } else {
        format!("{} warnings", snap.missing_includes_count)
    };

    // TODO / FIXME
    let todo_results = grep::search(&project.root, "TODO");
    let fixme_results = grep::search(&project.root, "FIXME");
    for r in todo_results.iter().chain(fixme_results.iter()) {
        snap.todos.push((r.file.clone(), r.line_no, r.text.trim().to_string()));
    }
    snap.todos_count = snap.todos.len();
    snap.todos_text = if snap.todos_count == 0 {
        "None".to_string()
    } else {
        format!("{}", snap.todos_count)
    };

    // Lint warnings
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        let paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("c"))
            .map(|e| e.path())
            .collect();
        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                let warnings = newc_core::lint::lint_file(&content);
                let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                for w in &warnings {
                    snap.lint_warnings.push((file_name.clone(), w.code, w.message.clone()));
                }
                snap.lint_count += warnings.len();
            }
        }
    }
    snap.lint_text = if snap.lint_count == 0 {
        "Clean".to_string()
    } else {
        format!("{} warnings", snap.lint_count)
    };

    state.health_snapshot = snap;
    state.health_computed = true;
}

#[derive(Default, Clone)]
pub struct HealthSnapshot {
    pub last_build_ok: bool,
    pub last_build_text: String,
    pub dead_code_count: usize,
    pub dead_code_text: String,
    pub dead_code_funcs: Vec<String>,
    pub missing_includes_count: usize,
    pub missing_includes_text: String,
    pub todos_count: usize,
    pub todos_text: String,
    pub todos: Vec<(String, usize, String)>,
    pub lint_count: usize,
    pub lint_text: String,
    pub lint_warnings: Vec<(String, &'static str, String)>,
}
