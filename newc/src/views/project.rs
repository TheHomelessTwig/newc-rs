use egui::{Color32, RichText, Ui};
use newc_core::project::Project;

pub enum ProjectAction {
    None,
    GoHome,
    AddModule,
    RemoveModule(String),
    SyncModule(String),
    SyncAll,
    Check,
    Tidy,
    RunMake(String),
    OpenInEditor,
    RefreshModules,
    OpenStats,
    OpenModuleDetail(String),
    OpenMainBuilder,
}

pub fn show(ui: &mut Ui, project: &Project, build_running: bool) -> ProjectAction {
    let mut action = ProjectAction::None;

    // Header row
    ui.horizontal(|ui| {
        if ui.button("← Home").clicked() {
            action = ProjectAction::GoHome;
        }
        ui.heading(&project.name);
        ui.label(
            RichText::new(project.root.display().to_string())
                .small()
                .color(Color32::GRAY),
        );
        if ui.button("Open in Editor").clicked() {
            action = ProjectAction::OpenInEditor;
        }
        if ui.button("Stats").clicked() {
            action = ProjectAction::OpenStats;
        }
        if ui
            .add(egui::Button::new("Compose main()").fill(Color32::from_rgb(60, 80, 140)))
            .on_hover_text("Visual main() composer — loads existing main.c structure")
            .clicked()
        {
            action = ProjectAction::OpenMainBuilder;
        }
    });
    ui.separator();

    // Build targets
    ui.horizontal(|ui| {
        ui.label("Make:");
        for target in ["all", "run", "debug", "release", "strict", "clean"] {
            if ui
                .add_enabled(!build_running, egui::Button::new(target))
                .clicked()
            {
                action = ProjectAction::RunMake(target.to_string());
            }
        }
        if build_running {
            ui.spinner();
        }
    });
    ui.separator();

    // Analysis tools
    ui.horizontal(|ui| {
        if ui.button("Check").on_hover_text("BFS reachability from main()").clicked() {
            action = ProjectAction::Check;
        }
        if ui.button("Tidy").on_hover_text("Remove unreachable functions").clicked() {
            action = ProjectAction::Tidy;
        }
        if ui.button("Sync All").on_hover_text("Regenerate all .h from .c").clicked() {
            action = ProjectAction::SyncAll;
        }
        if ui.button("Refresh").clicked() {
            action = ProjectAction::RefreshModules;
        }
    });
    ui.separator();

    // Module list
    ui.horizontal(|ui| {
        ui.heading("Modules");
        if ui.button("+ Add Module").clicked() {
            action = ProjectAction::AddModule;
        }
    });

    if project.modules.is_empty() {
        ui.label("No modules found.");
    } else {
        egui::Grid::new("modules_grid")
            .num_columns(5)
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Name").strong());
                ui.label(RichText::new("Functions").strong());
                ui.label(RichText::new("").strong());
                ui.label(RichText::new("").strong());
                ui.label(RichText::new("").strong());
                ui.end_row();

                for module in &project.modules {
                    ui.label(&module.name);
                    ui.label(module.function_count.to_string());
                    if ui.small_button("Edit").on_hover_text("Browse/edit functions in this module").clicked() {
                        action = ProjectAction::OpenModuleDetail(module.name.clone());
                    }
                    if ui.small_button("Sync").clicked() {
                        action = ProjectAction::SyncModule(module.name.clone());
                    }
                    if ui.small_button("Remove").on_hover_text("Delete this module").clicked() {
                        action = ProjectAction::RemoveModule(module.name.clone());
                    }
                    ui.end_row();
                }
            });
    }

    action
}
