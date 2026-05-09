use egui::{Color32, RichText, Ui};
use newc_core::{meta::ProjectMeta, project::Project};

pub enum ProjectAction {
    None,
    GoHome,
    AddModule,
    RemoveModule(String),
    ConfirmRemoveModule(String),
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
    ExportZip,
    OpenNotes,
    OpenGitPanel,
    SaveAsTemplate,
    OpenBuildHistory,
    OpenMetaEditor,
    OpenUsageTracker,
    OpenMakefile,
    OpenHealth,
    OpenSearch,
    ExportReport,
}

pub fn show(ui: &mut Ui, project: &Project, build_running: bool, meta: &ProjectMeta) -> ProjectAction {
    let mut action = ProjectAction::None;

    // Row 1: navigation + name
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
    });
    // Row 2: tools
    ui.horizontal_wrapped(|ui| {
        if ui.button("Stats").clicked() {
            action = ProjectAction::OpenStats;
        }
        if ui.button("Notes").on_hover_text("Project notepad").clicked() {
            action = ProjectAction::OpenNotes;
        }
        if ui.button("Git").on_hover_text("Git status, log, staging, push/pull").clicked() {
            action = ProjectAction::OpenGitPanel;
        }
        if ui.button("History").on_hover_text("Build history log").clicked() {
            action = ProjectAction::OpenBuildHistory;
        }
        if ui.button("Health").on_hover_text("Project health dashboard").clicked() {
            action = ProjectAction::OpenHealth;
        }
        if ui.button("Search").on_hover_text("Search across all source files").clicked() {
            action = ProjectAction::OpenSearch;
        }
        if ui.button("Usage").on_hover_text("Library function usage per file").clicked() {
            action = ProjectAction::OpenUsageTracker;
        }
        if ui.button("Makefile").on_hover_text("View/edit the project Makefile").clicked() {
            action = ProjectAction::OpenMakefile;
        }
        if ui.button("Export ZIP").on_hover_text("Bundle src/, include/, Makefile into a ZIP").clicked() {
            action = ProjectAction::ExportZip;
        }
        if ui.button("Export Report").on_hover_text("Generate Markdown project report").clicked() {
            action = ProjectAction::ExportReport;
        }
        if ui.button("Save as Template").on_hover_text("Save this project structure as a reusable template").clicked() {
            action = ProjectAction::SaveAsTemplate;
        }
        if ui
            .add(egui::Button::new("Compose main()").fill(Color32::from_rgb(60, 80, 140)))
            .on_hover_text("Visual main() composer — loads existing main.c structure")
            .clicked()
        {
            action = ProjectAction::OpenMainBuilder;
        }
    });

    // Metadata badge row
    if !meta.is_empty() {
        ui.horizontal_wrapped(|ui| {
            if !meta.course.is_empty() {
                ui.label(
                    RichText::new(format!("📚 {}", meta.course))
                        .small()
                        .color(Color32::from_rgb(100, 180, 255)),
                );
            }
            if !meta.assignment.is_empty() {
                ui.label(RichText::new(format!("📝 {}", meta.assignment)).small().color(Color32::LIGHT_GRAY));
            }
            if let Some(days) = meta.days_until_due() {
                let (text, color) = if days < 0 {
                    (format!("⚠ Overdue by {} days", -days), Color32::from_rgb(255, 80, 80))
                } else if days == 0 {
                    ("⚠ Due today".to_string(), Color32::from_rgb(255, 160, 0))
                } else if days <= 3 {
                    (format!("⏰ Due in {days} days"), Color32::from_rgb(255, 160, 0))
                } else {
                    (format!("📅 Due in {days} days ({})", meta.due_date), Color32::LIGHT_GRAY)
                };
                ui.label(RichText::new(text).small().color(color));
            }
            if let Some(m) = meta.marks_display() {
                ui.label(RichText::new(format!("🎯 {m}")).small().color(Color32::from_rgb(100, 220, 100)));
            }
            if ui.small_button("Edit Metadata").clicked() {
                action = ProjectAction::OpenMetaEditor;
            }
        });
    } else {
        ui.horizontal(|ui| {
            if ui.small_button("+ Add Metadata").on_hover_text("Add course, assignment, due date").clicked() {
                action = ProjectAction::OpenMetaEditor;
            }
        });
    }

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
                        action = ProjectAction::ConfirmRemoveModule(module.name.clone());
                    }
                    ui.end_row();
                }
            });
    }

    action
}
