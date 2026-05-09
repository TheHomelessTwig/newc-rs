use egui::{Color32, RichText, ScrollArea, Ui};
use newc_core::function_lib::FunctionLibrary;

pub fn show(
    ui: &mut Ui,
    lib: &FunctionLibrary,
    search: &mut String,
    selected: &mut Vec<String>,
) {
    ui.horizontal(|ui| {
        ui.label("Search:");
        ui.text_edit_singleline(search);
    });
    ui.separator();

    let candidates: Vec<_> = if search.is_empty() {
        lib.all().iter().collect()
    } else {
        lib.search(search)
    };

    // Resolved deps (shown inline)
    let resolved = lib.resolve_deps(selected);

    ScrollArea::vertical().id_salt("func_picker").max_height(300.0).show(ui, |ui| {
        let mut modules: Vec<String> = candidates.iter().map(|f| f.module.clone()).collect();
        modules.sort();
        modules.dedup();

        for module in &modules {
            ui.collapsing(module.as_str(), |ui| {
                for func in candidates.iter().filter(|f| &f.module == module) {
                    let is_dep = resolved.contains(&func.name) && !selected.contains(&func.name);
                    let mut checked = selected.contains(&func.name) || is_dep;

                    ui.horizontal(|ui| {
                        let response = ui.checkbox(&mut checked, "");
                        if response.changed() {
                            if checked {
                                if !selected.contains(&func.name) {
                                    selected.push(func.name.clone());
                                }
                            } else {
                                selected.retain(|n| n != &func.name);
                            }
                        }
                        if is_dep {
                            ui.label(
                                RichText::new(&func.name)
                                    .color(Color32::from_rgb(150, 150, 255))
                                    .monospace(),
                            )
                            .on_hover_text("Auto-selected as a dependency");
                        } else {
                            ui.label(RichText::new(&func.name).monospace());
                        }
                        ui.label(
                            RichText::new(&func.description)
                                .color(Color32::GRAY)
                                .small(),
                        );
                    });

                    if !func.tags.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            for tag in &func.tags {
                                ui.label(
                                    RichText::new(tag)
                                        .small()
                                        .color(Color32::from_rgb(100, 200, 150)),
                                );
                            }
                        });
                    }
                }
            });
        }
    });

    if !selected.is_empty() {
        ui.separator();
        ui.label(format!(
            "{} selected ({} with deps)",
            selected.len(),
            resolved.len()
        ));
    }
}
