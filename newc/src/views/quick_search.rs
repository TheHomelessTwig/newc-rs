use egui::{Color32, Key, RichText};
use newc_core::function_lib::FunctionLibrary;
use std::path::PathBuf;

#[derive(Default)]
pub struct QuickSearchState {
    pub open: bool,
    pub query: String,
    pub cursor: usize,
}

#[derive(Clone)]
pub enum QuickSearchResult {
    Function { name: String, module: String, description: String },
    Project { name: String, path: PathBuf },
}

pub enum QuickSearchAction {
    None,
    OpenFunction(String),
    OpenProject(PathBuf),
    Close,
}

pub fn handle_shortcut(ctx: &egui::Context, state: &mut QuickSearchState) {
    ctx.input(|i| {
        if i.key_pressed(Key::P) && i.modifiers.ctrl {
            state.open = !state.open;
            if state.open {
                state.query.clear();
                state.cursor = 0;
            }
        }
        if i.key_pressed(Key::Escape) && state.open {
            state.open = false;
        }
    });
}

pub fn show(
    ctx: &egui::Context,
    state: &mut QuickSearchState,
    lib: &FunctionLibrary,
    projects: &[PathBuf],
) -> QuickSearchAction {
    if !state.open {
        return QuickSearchAction::None;
    }

    let mut action = QuickSearchAction::None;

    // Collect results
    let results = collect_results(&state.query, lib, projects);

    egui::Window::new("Quick Search")
        .title_bar(false)
        .resizable(false)
        .fixed_size([520.0, 400.0])
        .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
        .show(ctx, |ui| {
            // Search bar
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.query)
                    .desired_width(f32::INFINITY)
                    .hint_text("Search functions, projects… (Ctrl+P to toggle, Esc to close)")
                    .font(egui::TextStyle::Heading),
            );
            // Auto-focus on open
            resp.request_focus();

            // Handle up/down arrow keys
            ui.input(|i| {
                if i.key_pressed(Key::ArrowDown) && state.cursor + 1 < results.len() {
                    state.cursor += 1;
                }
                if i.key_pressed(Key::ArrowUp) && state.cursor > 0 {
                    state.cursor -= 1;
                }
            });

            // Reset cursor when query changes
            if resp.changed() {
                state.cursor = 0;
            }

            ui.separator();

            egui::ScrollArea::vertical().max_height(330.0).show(ui, |ui| {
                if results.is_empty() {
                    ui.label(RichText::new("No results").color(Color32::GRAY));
                }

                for (i, result) in results.iter().enumerate() {
                    let selected = i == state.cursor;
                    let bg = if selected {
                        Color32::from_rgba_premultiplied(60, 100, 180, 80)
                    } else {
                        Color32::TRANSPARENT
                    };

                    egui::Frame::new().fill(bg).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let clicked = match result {
                            QuickSearchResult::Function { name, module, description } => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(name).strong().monospace());
                                    ui.label(
                                        RichText::new(format!("({module})"))
                                            .small()
                                            .color(Color32::GRAY),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(description)
                                                    .small()
                                                    .color(Color32::LIGHT_GRAY),
                                            );
                                        },
                                    );
                                })
                                .response
                                .interact(egui::Sense::click())
                                .clicked()
                            }
                            QuickSearchResult::Project { name, path } => {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("📁 ").color(Color32::from_rgb(200, 160, 80)),
                                    );
                                    ui.label(RichText::new(name).strong());
                                    ui.label(
                                        RichText::new(path.display().to_string())
                                            .small()
                                            .color(Color32::GRAY),
                                    );
                                })
                                .response
                                .interact(egui::Sense::click())
                                .clicked()
                            }
                        };

                        // Enter key activates cursor item; click activates hovered item
                        let activated = clicked
                            || (selected
                                && ui.input(|i| i.key_pressed(Key::Enter)));

                        if activated {
                            action = match result.clone() {
                                QuickSearchResult::Function { name, .. } => {
                                    state.open = false;
                                    QuickSearchAction::OpenFunction(name)
                                }
                                QuickSearchResult::Project { path, .. } => {
                                    state.open = false;
                                    QuickSearchAction::OpenProject(path)
                                }
                            };
                        }
                    });
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("↑↓ navigate  Enter select  Esc close").small().color(Color32::GRAY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} results", results.len())).small().color(Color32::GRAY));
                });
            });
        });

    action
}

fn collect_results(
    query: &str,
    lib: &FunctionLibrary,
    projects: &[PathBuf],
) -> Vec<QuickSearchResult> {
    let q = query.to_lowercase();
    let mut results = Vec::new();

    // Functions
    let funcs: Vec<_> = if q.is_empty() {
        lib.all().iter().collect()
    } else {
        lib.search(&q)
    };
    for f in funcs.iter().take(20) {
        results.push(QuickSearchResult::Function {
            name: f.name.clone(),
            module: f.module.clone(),
            description: f.description.clone(),
        });
    }

    // Projects
    for path in projects {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if q.is_empty() || name.to_lowercase().contains(&q) || path.display().to_string().to_lowercase().contains(&q) {
            results.push(QuickSearchResult::Project {
                name,
                path: path.clone(),
            });
        }
    }

    results
}
