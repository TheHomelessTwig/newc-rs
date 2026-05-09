use std::path::PathBuf;

use egui::{CentralPanel, Color32, Context, RichText, SidePanel, TopBottomPanel};
use newc_core::{
    analysis,
    export,
    header,
    main_builder::MainBuilderState,
    module,
    project::Project,
    project_template,
    scaffold::{DefaultModule, ScaffoldOptions, create_project},
    sync::{self, extract_function_implementations, update_function_in_source},
};

use crate::build_runner::{BuildLine, BuildRunner, LineKind};
use crate::state::{AppState, BuildState, View};
use crate::views;
use crate::views::import_c::ImportState;

pub struct NewcApp {
    state: AppState,
    runner: BuildRunner,
    build_panel_open: bool,
    build_auto_scroll: bool,
}

impl NewcApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();
        let scan_paths = state.config.scan_paths();
        state.known_projects = Project::discover(&scan_paths);

        let runner = BuildRunner::spawn(cc.egui_ctx.clone());
        Self { state, runner, build_panel_open: true, build_auto_scroll: true }
    }

    fn drain_build_output(&mut self) {
        for line in self.runner.drain() {
            if let LineKind::Done { exit_code } = line.kind {
                self.state.build_state = BuildState::Done { exit_code };
                self.state.build_lines.push(line);
            } else {
                self.state.build_state = BuildState::Running;
                self.state.build_lines.push(line);
            }
        }
    }

    fn open_project(&mut self, path: PathBuf) {
        match Project::open(path.clone()) {
            Ok(p) => {
                if !self.state.known_projects.contains(&path) {
                    self.state.known_projects.push(path);
                }
                self.state.cached_stats = None;
                self.state.view = View::ProjectDetail(p);
            }
            Err(e) => self.state.set_error(e.to_string()),
        }
    }
}

impl eframe::App for NewcApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.drain_build_output();

        // Handle Ctrl+P quick search shortcut
        views::quick_search::handle_shortcut(ctx, &mut self.state.quick_search);

        // Top bar
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let is_home = matches!(self.state.view, View::Home);
                let is_create = matches!(self.state.view, View::CreateProject);
                let is_lib = matches!(self.state.view, View::FunctionLibrary);
                let is_settings = matches!(self.state.view, View::Settings);
                if ui.selectable_label(is_home, "Home").clicked() {
                    self.state.view = View::Home;
                }
                if ui.selectable_label(is_create, "New Project").clicked() {
                    self.state.view = View::CreateProject;
                }
                if ui.selectable_label(is_lib, "Function Library").clicked() {
                    self.state.view = View::FunctionLibrary;
                }
                if ui.selectable_label(is_settings, "Settings").clicked() {
                    self.state.view = View::Settings;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.toggle_value(&mut self.build_panel_open, "Build Output");
                    ui.label(
                        RichText::new("Ctrl+P")
                            .small()
                            .color(Color32::GRAY),
                    );
                    if let Some((msg, t)) = &self.state.status {
                        if t.elapsed().as_secs() < 4 {
                            ui.label(RichText::new(msg).color(Color32::from_rgb(100, 220, 100)));
                        } else {
                            self.state.status = None;
                        }
                    }
                });
            });
        });

        // Bottom build panel
        if self.build_panel_open {
            TopBottomPanel::bottom("build_panel")
                .min_height(150.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.small_button("Clear").clicked() {
                            self.state.build_lines.clear();
                            self.state.build_state = BuildState::Idle;
                        }
                        match &self.state.build_state {
                            BuildState::Running => {
                                if ui.small_button("Kill").clicked() {
                                    self.runner.kill();
                                }
                            }
                            BuildState::Done { exit_code: Some(0) } => {
                                ui.label(RichText::new("✓ OK").color(Color32::from_rgb(100, 220, 100)));
                            }
                            BuildState::Done { .. } => {
                                ui.label(RichText::new("✗ Failed").color(Color32::from_rgb(255, 80, 80)));
                            }
                            BuildState::Idle => {}
                        }
                    });
                    views::build_panel::show(
                        ui,
                        &self.state.build_lines,
                        &mut self.build_auto_scroll,
                    );
                });
        }

        // Left sidebar — project list
        SidePanel::left("sidebar")
            .min_width(180.0)
            .max_width(280.0)
            .show(ctx, |ui| {
                ui.label(RichText::new("Projects").strong());
                ui.separator();
                let projects = self.state.known_projects.clone();
                for path in &projects {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string());
                    let active = self
                        .state
                        .current_project()
                        .map(|p| &p.root == path)
                        .unwrap_or(false);
                    if ui.selectable_label(active, &name).clicked() {
                        self.open_project(path.clone());
                    }
                }
            });

        // Error modal
        if self.state.error_msg.is_some() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(self.state.error_msg.as_deref().unwrap_or(""));
                    if ui.button("OK").clicked() {
                        self.state.error_msg = None;
                    }
                });
        }

        // Tidy confirm modal
        if self.state.show_tidy_confirm {
            let candidates = self.state.tidy_candidates.clone();
            egui::Window::new("Confirm Tidy")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("The following unreachable functions will be removed:");
                    for f in &candidates {
                        ui.label(format!("  {f}"));
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Remove").clicked() {
                            self.state.show_tidy_confirm = false;
                            if let Some(project) = self.state.current_project().cloned() {
                                match analysis::tidy(&project.root, &candidates) {
                                    Ok(log) => {
                                        for line in &log {
                                            self.state.build_lines.push(BuildLine {
                                                text: line.clone(),
                                                kind: LineKind::Info,
                                            });
                                        }
                                        self.state.set_status("Tidy complete.");
                                        if let View::ProjectDetail(ref mut p) = self.state.view {
                                            let _ = p.refresh_modules();
                                        }
                                    }
                                    Err(e) => self.state.set_error(e.to_string()),
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.state.show_tidy_confirm = false;
                        }
                    });
                });
        }

        // Import from .c modal
        if self.state.show_import {
            egui::Window::new("Import from .c")
                .resizable(true)
                .min_width(500.0)
                .show(ctx, |ui| {
                    let action =
                        views::import_c::show(ui, &mut self.state.import_state);
                    match action {
                        views::import_c::ImportAction::PickFile => {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("C source", &["c"])
                                .pick_file()
                            {
                                let label = path.display().to_string();
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    let extracted = extract_function_implementations(&content);
                                    let n = extracted.len();
                                    self.state.import_state = ImportState {
                                        extracted,
                                        selected: vec![true; n],
                                        target_module: path
                                            .file_stem()
                                            .map(|s| s.to_string_lossy().into_owned())
                                            .unwrap_or_default(),
                                        path_label: label,
                                    };
                                }
                            }
                        }
                        views::import_c::ImportAction::Import(funcs) => {
                            for func in &funcs {
                                self.state.function_lib.upsert(func.clone());
                                let _ = newc_core::function_lib::FunctionLibrary::save_user_function(func);
                            }
                            self.state.set_status(format!("Imported {} function(s).", funcs.len()));
                            self.state.show_import = false;
                            self.state.import_state = ImportState::default();
                        }
                        views::import_c::ImportAction::Cancel => {
                            self.state.show_import = false;
                            self.state.import_state = ImportState::default();
                        }
                        views::import_c::ImportAction::None => {}
                    }
                });
        }

        // New group modal
        if self.state.show_new_group {
            egui::Window::new("New Function Group")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("new_group_grid").num_columns(2).show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.state.new_group_name);
                        ui.end_row();
                        ui.label("Description:");
                        ui.text_edit_singleline(&mut self.state.new_group_desc);
                        ui.end_row();
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let valid = !self.state.new_group_name.trim().is_empty();
                        if ui.add_enabled(valid, egui::Button::new("Create")).clicked() {
                            let name = self.state.new_group_name.trim().to_string();
                            let desc = self.state.new_group_desc.trim().to_string();
                            if self.state.function_lib.create_group(&name, &desc) {
                                let _ = self.state.function_lib.save_groups();
                                self.state.library_state.active_group = Some(name.clone());
                                self.state.set_status(format!("Group '{name}' created."));
                            } else {
                                self.state.set_error(format!("Group '{name}' already exists."));
                            }
                            self.state.show_new_group = false;
                            self.state.new_group_name.clear();
                            self.state.new_group_desc.clear();
                        }
                        if ui.button("Cancel").clicked() {
                            self.state.show_new_group = false;
                            self.state.new_group_name.clear();
                        }
                    });
                });
        }

        // Group action modal (rename / delete)
        if let Some(target) = self.state.group_action_target.clone() {
            egui::Window::new(format!("Group: {target}"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Rename").strong());
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.state.group_rename_input);
                        let valid = !self.state.group_rename_input.trim().is_empty()
                            && self.state.group_rename_input != target;
                        if ui.add_enabled(valid, egui::Button::new("Rename")).clicked() {
                            let new_name = self.state.group_rename_input.trim().to_string();
                            if self.state.function_lib.rename_group(&target, &new_name) {
                                // Persist renamed functions
                                let funcs: Vec<_> = self.state.function_lib
                                    .by_module(&new_name)
                                    .into_iter()
                                    .cloned()
                                    .collect();
                                for f in &funcs {
                                    let _ = newc_core::function_lib::FunctionLibrary::save_user_function(f);
                                }
                                let _ = self.state.function_lib.save_groups();
                                self.state.library_state.active_group = Some(new_name.clone());
                                self.state.set_status(format!("Renamed to '{new_name}'."));
                            } else {
                                self.state.set_error("Rename failed — name may already exist.".to_string());
                            }
                            self.state.group_action_target = None;
                        }
                    });
                    ui.separator();
                    ui.label(RichText::new("Delete group").strong());
                    ui.checkbox(&mut self.state.delete_group_cascade, "Also delete all functions in this group");
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Button::new("Delete").fill(Color32::from_rgb(160, 40, 40)))
                            .clicked()
                        {
                            let cascade = self.state.delete_group_cascade;
                            self.state.function_lib.delete_group(&target, cascade);
                            let _ = self.state.function_lib.save_groups();
                            self.state.set_status(format!("Group '{target}' deleted."));
                            self.state.library_state.active_group = None;
                            self.state.group_action_target = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.state.group_action_target = None;
                        }
                    });
                });
        }

        // Quick search overlay
        let qs_action = views::quick_search::show(
            ctx,
            &mut self.state.quick_search,
            &self.state.function_lib,
            &self.state.known_projects,
        );
        match qs_action {
            views::quick_search::QuickSearchAction::OpenFunction(name) => {
                self.state.library_state.selected = Some(name);
                self.state.view = View::FunctionLibrary;
            }
            views::quick_search::QuickSearchAction::OpenProject(path) => {
                self.open_project(path);
            }
            views::quick_search::QuickSearchAction::Close
            | views::quick_search::QuickSearchAction::None => {}
        }

        // Central panel
        CentralPanel::default().show(ctx, |ui| {
            let view = self.state.view.clone();
            match view {
                View::Settings => {
                    let action = views::settings::show(ui, &mut self.state.config_draft);
                    if let views::settings::SettingsAction::Save(cfg) = action {
                        self.state.config = cfg.clone();
                        self.state.config_draft = cfg.clone();
                        if let Err(e) = cfg.save() {
                            self.state.set_error(e.to_string());
                        } else {
                            // Rescan with updated dirs
                            let scan_paths = self.state.config.scan_paths();
                            let found = Project::discover(&scan_paths);
                            for p in found {
                                if !self.state.known_projects.contains(&p) {
                                    self.state.known_projects.push(p);
                                }
                            }
                            self.state.set_status("Settings saved.");
                        }
                    }
                }

                View::FunctionLibrary => {
                    if self.state.show_import {
                        return; // import modal is shown as a Window overlay
                    }
                    let action = views::library::show(
                        ui,
                        &self.state.function_lib,
                        &mut self.state.library_state,
                    );
                    match action {
                        views::library::LibraryAction::Save(func) => {
                            self.state.function_lib.upsert(func.clone());
                            if let Err(e) = newc_core::function_lib::FunctionLibrary::save_user_function(&func) {
                                self.state.set_error(e.to_string());
                            } else {
                                self.state.set_status(format!("Saved '{}'.", func.name));
                            }
                        }
                        views::library::LibraryAction::Delete(name) => {
                            self.state.function_lib.remove(&name);
                            delete_user_function(&name, &self.state.function_lib);
                            self.state.set_status(format!("Deleted '{name}'."));
                        }
                        views::library::LibraryAction::UpdateNotes { name, notes } => {
                            if let Some(f) = self.state.function_lib.get_mut(&name) {
                                f.notes = notes;
                                let func = f.clone();
                                let _ = newc_core::function_lib::FunctionLibrary::save_user_function(&func);
                            }
                        }
                        views::library::LibraryAction::OpenImport => {
                            self.state.show_import = true;
                        }
                        views::library::LibraryAction::CreateGroup { name, .. } => {
                            if name == "__OPEN_DIALOG__" {
                                // Open the new-group modal
                                self.state.show_new_group = true;
                                self.state.new_group_name.clear();
                                self.state.new_group_desc.clear();
                            } else {
                                // Direct create (no dialog needed)
                                if self.state.function_lib.create_group(&name, "") {
                                    let _ = self.state.function_lib.save_groups();
                                    self.state.set_status(format!("Group '{name}' created."));
                                }
                            }
                        }
                        views::library::LibraryAction::RenameGroup { old, new } => {
                            if old == "__OPEN_DIALOG__" {
                                // Open group action modal for `new` (current group name)
                                self.state.group_action_target = Some(new.clone());
                                self.state.group_rename_input = new;
                                self.state.delete_group_cascade = false;
                            } else {
                                if self.state.function_lib.rename_group(&old, &new) {
                                    let _ = self.state.function_lib.save_groups();
                                    self.state.set_status(format!("Renamed '{old}' → '{new}'."));
                                }
                            }
                        }
                        views::library::LibraryAction::DeleteGroup { name, cascade } => {
                            self.state.function_lib.delete_group(&name, cascade);
                            let _ = self.state.function_lib.save_groups();
                            self.state.set_status(format!("Group '{name}' deleted."));
                            self.state.library_state.active_group = None;
                        }
                        views::library::LibraryAction::None => {}
                    }
                }

                View::Home => {
                    let action = views::home::show(ui, &self.state.known_projects);
                    if action.go_create {
                        self.state.view = View::CreateProject;
                    }
                    if let Some(path) = action.open_project {
                        self.open_project(path);
                    }
                    if let Some(i) = action.remove_project {
                        self.state.known_projects.remove(i);
                    }
                    if action.browse_for_project {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.open_project(path);
                        }
                    }
                }

                View::CreateProject => {
                    let action = views::create::show(
                        ui,
                        &mut self.state.create_name,
                        &mut self.state.create_git,
                        &mut self.state.create_author,
                        &mut self.state.create_include_input,
                        &mut self.state.create_include_math,
                        &mut self.state.create_include_display,
                        &mut self.state.create_include_array,
                        &mut self.state.func_search,
                        &mut self.state.func_selected,
                        &self.state.function_lib,
                        &mut false,
                        &mut self.state.selected_template,
                    );
                    match action {
                        views::create::CreateAction::Create {
                            name, git, author,
                            include_input, include_math, include_display, include_array,
                            location,
                        } => {
                            let mut modules = Vec::new();
                            if include_input { modules.push(DefaultModule::Input); }
                            if include_math { modules.push(DefaultModule::Math); }
                            if include_display { modules.push(DefaultModule::Display); }
                            if include_array { modules.push(DefaultModule::Array); }

                            // Remember selected template to seed composer after project created
                            let template_idx = self.state.selected_template;

                            let opts = ScaffoldOptions {
                                name: name.clone(),
                                git_init: git,
                                author,
                                modules,
                            };
                            match create_project(&opts, &location) {
                                Ok(()) => {
                                    let root = location.join(&name);
                                    // Seed composer from template if one was selected
                                    if let Some(idx) = template_idx {
                                        let templates = project_template::all_templates();
                                        if let Some(t) = templates.get(idx) {
                                            self.state.main_builder = (t.builder)();
                                            self.state.composer_undo.clear();
                                            self.state.composer_redo.clear();
                                        }
                                    }
                                    self.open_project(root);
                                    self.state.set_status(format!("Project '{name}' created."));
                                    self.state.create_name.clear();
                                    self.state.selected_template = None;
                                }
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::create::CreateAction::Cancel => {
                            self.state.view = View::Home;
                        }
                        views::create::CreateAction::None => {}
                    }
                }

                View::ProjectStats(ref project) => {
                    let project = project.clone();
                    if ui.button("← Project").clicked() {
                        self.state.view = View::ProjectDetail(project.clone());
                        return;
                    }
                    let stats = self.state.get_or_compute_stats().cloned();
                    if let Some(s) = stats {
                        views::stats::show(ui, &project, &s);
                    }
                }

                View::ProjectDetail(ref project) => {
                    let project = project.clone();
                    let build_running = self.state.build_state == BuildState::Running;
                    let action = views::project::show(ui, &project, build_running);

                    match action {
                        views::project::ProjectAction::GoHome => {
                            self.state.view = View::Home;
                        }
                        views::project::ProjectAction::OpenStats => {
                            self.state.cached_stats = None;
                            self.state.view = View::ProjectStats(project);
                        }
                        views::project::ProjectAction::AddModule => {
                            self.state.add_module_name.clear();
                            self.state.func_search.clear();
                            self.state.func_selected.clear();
                            self.state.view = View::AddModule { project: project.clone() };
                        }
                        views::project::ProjectAction::RemoveModule(name) => {
                            match module::remove_module(&project.root, &name) {
                                Ok(()) => {
                                    self.state.set_status(format!("Removed module '{name}'."));
                                    if let View::ProjectDetail(ref mut p) = self.state.view {
                                        let _ = p.refresh_modules();
                                    }
                                }
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::project::ProjectAction::SyncModule(name) => {
                            match sync::sync_module(&project.root, &name) {
                                Ok(()) => self.state.set_status(format!("Synced {name}.h")),
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::project::ProjectAction::SyncAll => {
                            match sync::sync_all(&project.root) {
                                Ok(msgs) => {
                                    for m in &msgs {
                                        self.state.build_lines.push(BuildLine {
                                            text: m.clone(),
                                            kind: LineKind::Info,
                                        });
                                    }
                                    self.state.set_status("Sync complete.");
                                }
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::project::ProjectAction::Check => {
                            match analysis::check(&project.root) {
                                Ok(unreachable) => {
                                    if unreachable.is_empty() {
                                        self.state.build_lines.push(BuildLine {
                                            text: "All module functions are reachable from main.".into(),
                                            kind: LineKind::Info,
                                        });
                                    } else {
                                        self.state.build_lines.push(BuildLine {
                                            text: format!("{} unreachable function(s):", unreachable.len()),
                                            kind: LineKind::Stderr,
                                        });
                                        for f in &unreachable {
                                            self.state.build_lines.push(BuildLine {
                                                text: format!("  {} ({})", f.name, f.source.display()),
                                                kind: LineKind::Stderr,
                                            });
                                        }
                                    }
                                    self.build_panel_open = true;
                                }
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::project::ProjectAction::Tidy => {
                            match analysis::check(&project.root) {
                                Ok(unreachable) if unreachable.is_empty() => {
                                    self.state.set_status("Nothing to tidy.");
                                }
                                Ok(unreachable) => {
                                    self.state.tidy_candidates =
                                        unreachable.iter().map(|f| f.name.clone()).collect();
                                    self.state.show_tidy_confirm = true;
                                }
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::project::ProjectAction::RunMake(target) => {
                            self.state.build_state = BuildState::Running;
                            self.build_panel_open = true;
                            self.runner.run(&target, project.root.clone());
                        }
                        views::project::ProjectAction::OpenInEditor => {
                            self.state.config.open_in_editor(&project.root);
                        }
                        views::project::ProjectAction::RefreshModules => {
                            if let View::ProjectDetail(ref mut p) = self.state.view {
                                let _ = p.refresh_modules();
                            }
                        }
                        views::project::ProjectAction::OpenModuleDetail(module_name) => {
                            self.state.module_detail_state = Default::default();
                            self.state.view = View::ModuleDetail {
                                project: project.clone(),
                                module_name,
                            };
                        }
                        views::project::ProjectAction::OpenMainBuilder => {
                            self.state.main_builder = MainBuilderState::load_from_main_c(&project.root);
                            self.state.composer_undo.clear();
                            self.state.composer_redo.clear();
                            self.state.view = View::MainBuilder(project.clone());
                        }
                        views::project::ProjectAction::ExportZip => {
                            let parent = project.root.parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| std::path::PathBuf::from("."));
                            let dest = if let Some(d) = rfd::FileDialog::new()
                                .set_file_name(&format!("{}.zip", project.name))
                                .set_directory(&parent)
                                .save_file()
                            { d.parent().map(|p| p.to_path_buf()).unwrap_or(parent) } else { return };
                            match export::export_zip(&project.root, &project.name, &dest) {
                                Ok(path) => self.state.set_status(format!("Exported to {}", path.display())),
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::project::ProjectAction::None => {}
                    }
                }

                View::ModuleDetail { ref project, ref module_name } => {
                    let project = project.clone();
                    let module_name = module_name.clone();
                    let src_path = project.root.join("src").join(format!("{module_name}.c"));
                    let lib = self.state.function_lib.clone();
                    let action = views::module_detail::show(
                        ui,
                        &module_name,
                        &src_path,
                        &mut self.state.module_detail_state,
                        &lib,
                    );
                    match action {
                        views::module_detail::ModuleDetailAction::GoBack => {
                            self.state.view = View::ProjectDetail(project);
                        }
                        views::module_detail::ModuleDetailAction::SaveFunction { name, new_impl } => {
                            match update_function_in_source(&src_path, &name, &new_impl) {
                                Ok(()) => self.state.set_status(format!("Saved {name}.")),
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::module_detail::ModuleDetailAction::DeleteFunction(name) => {
                            match analysis::tidy(&project.root, &[name.clone()]) {
                                Ok(_) => self.state.set_status(format!("Deleted {name}.")),
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::module_detail::ModuleDetailAction::AddFromLibrary => {
                            self.state.func_search.clear();
                            self.state.func_selected.clear();
                            self.state.add_module_name = module_name.clone();
                            self.state.view = View::AddModule { project };
                        }
                        views::module_detail::ModuleDetailAction::OpenHeaderEditor => {
                            self.state.header_editor_state = Default::default();
                            self.state.view = View::HeaderEditor { project, module_name };
                        }
                        views::module_detail::ModuleDetailAction::None => {}
                    }
                }

                View::HeaderEditor { ref project, ref module_name } => {
                    let project = project.clone();
                    let module_name = module_name.clone();
                    let hdr_path = project.root.join("include").join(format!("{module_name}.h"));
                    let action = views::header_editor::show(
                        ui,
                        &hdr_path,
                        &module_name,
                        &mut self.state.header_editor_state,
                    );
                    match action {
                        views::header_editor::HeaderEditorAction::Save => {
                            match header::write_ignore_block(&hdr_path, &self.state.header_editor_state.content) {
                                Ok(()) => {
                                    self.state.set_status("Header saved.");
                                    self.state.view = View::ModuleDetail { project, module_name };
                                    self.state.header_editor_state = Default::default();
                                }
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::header_editor::HeaderEditorAction::Close => {
                            self.state.view = View::ModuleDetail { project, module_name };
                            self.state.header_editor_state = Default::default();
                        }
                        views::header_editor::HeaderEditorAction::None => {}
                    }
                }

                View::MainBuilder(ref project) => {
                    let project = project.clone();
                    let author = self.state.create_author.clone();
                    let action = views::main_builder::show(
                        ui,
                        &project,
                        &mut self.state.main_builder,
                        &author,
                        &mut self.state.composer_undo,
                        &mut self.state.composer_redo,
                    );
                    match action {
                        views::main_builder::BuilderAction::GoBack => {
                            self.state.view = View::ProjectDetail(project);
                        }
                        views::main_builder::BuilderAction::WriteMainC => {
                            let date = chrono::Local::now().format("%d/%m/%Y").to_string();
                            let code = self.state.main_builder.preview(&author, &date);
                            let main_c = project.root.join("src").join("main.c");
                            match std::fs::write(&main_c, code) {
                                Ok(()) => self.state.set_status("Written to src/main.c"),
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::main_builder::BuilderAction::None
                        | views::main_builder::BuilderAction::Snapshot => {}
                    }
                }

                View::AddModule { ref project, .. } => {
                    let project = project.clone();
                    let lib = self.state.function_lib.clone();
                    let action = views::add_module::show(
                        ui,
                        &project,
                        &mut self.state.add_module_name,
                        &mut self.state.func_search,
                        &mut self.state.func_selected,
                        &lib,
                    );
                    match action {
                        views::add_module::AddModuleAction::Create { name, selected_funcs } => {
                            match module::add_module(&project.root, &name) {
                                Ok(()) => {
                                    if !selected_funcs.is_empty() {
                                        inject_functions_into_module(
                                            &project.root,
                                            &name,
                                            &selected_funcs,
                                            &lib,
                                        );
                                    }
                                    self.state.set_status(format!("Module '{name}' created."));
                                    match Project::open(project.root.clone()) {
                                        Ok(p) => self.state.view = View::ProjectDetail(p),
                                        Err(e) => self.state.set_error(e.to_string()),
                                    }
                                }
                                Err(e) => self.state.set_error(e.to_string()),
                            }
                        }
                        views::add_module::AddModuleAction::Cancel => {
                            self.state.view = View::ProjectDetail(project);
                        }
                        views::add_module::AddModuleAction::None => {}
                    }
                }
            }
        });
    }
}

fn inject_functions_into_module(
    root: &std::path::Path,
    module_name: &str,
    selected: &[String],
    lib: &newc_core::function_lib::FunctionLibrary,
) {
    let resolved = lib.resolve_deps(selected);
    let src_path = root.join("src").join(format!("{module_name}.c"));
    let hdr_path = root.join("include").join(format!("{module_name}.h"));

    let mut src_additions = String::new();
    let mut hdr_additions = String::new();

    for fname in &resolved {
        if let Some(func) = lib.all().iter().find(|f| &f.name == fname) {
            src_additions.push_str(&func.impl_code);
            src_additions.push('\n');
            hdr_additions.push_str(&func.header_code);
            hdr_additions.push('\n');
        }
    }

    if let Ok(mut content) = std::fs::read_to_string(&src_path) {
        content.push_str(&src_additions);
        let _ = std::fs::write(&src_path, content);
    }

    if let Ok(content) = std::fs::read_to_string(&hdr_path) {
        let new_content = content.replace("#endif", &format!("{hdr_additions}\n#endif"));
        let _ = std::fs::write(&hdr_path, new_content);
    }
}

fn delete_user_function(name: &str, lib: &newc_core::function_lib::FunctionLibrary) {
    let Some(dir) = dirs::config_dir().map(|d| d.join("newc").join("functions")) else {
        return;
    };
    if !dir.exists() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("toml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else { continue };
        if !content.contains(name) {
            continue;
        }
        let module = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let remaining: Vec<_> = lib.by_module(&module).into_iter().cloned().collect();
        if remaining.is_empty() {
            let _ = std::fs::remove_file(&path);
        } else {
            for func in &remaining {
                let _ = newc_core::function_lib::FunctionLibrary::save_user_function(func);
            }
        }
        break;
    }
}
