//! Top-level iced application — [`NewcApp`] owns all mutable state and
//! implements the iced daemon callbacks (`new`, `update`, `view_for_window`,
//! `title_for_window`, `theme_for_window`, `subscription`).
//!
//! The daemon is started by [`crate::run_gui_inline`] and manages one mandatory
//! main window plus up to three optional detached windows (Function Library,
//! C Reference, Snippets). Each window is identified by an [`iced::window::Id`]
//! stored in [`crate::state::AppState`].

use std::path::PathBuf;
use std::time::Duration;
use notify::Watcher;

use iced::{Element, Subscription, Task, Theme};
use iced::widget::{column, row, text, button, container, scrollable, Space, Stack};
use iced::{Alignment, Background, Length, Color};
use crate::theme as th;
use newc_core::{
    function_lib::FunctionLibrary,
    build_history::{self, BuildRecord},
    diag,
    meta,
    module,
    notes,
    project::Project,
    scaffold::{DefaultModule, ScaffoldOptions, create_project},
};

use crate::build_runner::{BuildRunner, LineKind};
use crate::state::{AppState, BuildState, Message, View, save_known_projects};
use crate::views;

/// Root application struct passed to `iced::daemon`.
///
/// Holds the full UI state, the background build runner, and the user function
/// library. All three are initialized once in [`NewcApp::new`] and updated
/// exclusively through [`NewcApp::update`].
pub struct NewcApp {
    state: AppState,
    runner: BuildRunner,
    function_lib: FunctionLibrary,
    /// Lazily spawned on first hover request for a project; `None` if
    /// `clangd` isn't installed or hasn't been requested yet.
    lsp_client: Option<crate::lsp::LspClient>,
    lsp_opened_modules: std::collections::HashSet<String>,
}

impl NewcApp {
    /// Initialize the application and open the main window.
    ///
    /// Loads persisted state (known projects, config, recent projects), merges
    /// any projects discovered via configured scan paths, and then opens the
    /// main 1200×780 window. If `initial_path` points to a valid newc project
    /// it is opened immediately; otherwise the first-run onboarding wizard is
    /// shown when no `.onboarded` marker exists.
    pub fn new(initial_path: Option<PathBuf>) -> (Self, Task<Message>) {
        let mut state = AppState::new();

        // Merge discovered projects
        let scan_paths = state.config.scan_paths();
        let discovered = Project::discover(&scan_paths);
        for p in discovered {
            if !state.known_projects.contains(&p) {
                state.known_projects.push(p);
            }
        }

        // Open the main window (daemon mode has no automatic window)
        let (main_id, open_task) = iced::window::open(iced::window::Settings {
            size: iced::Size::new(1200.0, 780.0),
            min_size: Some(iced::Size::new(900.0, 550.0)),
            position: iced::window::Position::Default,
            ..Default::default()
        });
        state.main_window = Some(main_id);

        let runner = BuildRunner::spawn();
        let function_lib = FunctionLibrary::load();

        let mut app = Self {
            state, runner, function_lib,
            lsp_client: None,
            lsp_opened_modules: std::collections::HashSet::new(),
        };

        if let Some(path) = initial_path {
            app.open_project(path);
        } else if app.state.is_first_run {
            app.state.view = View::Onboarding(0);
            app.state.onboarding_found = scan_for_onboarding_projects();
        }

        (app, open_task.discard())
    }

    /// Return the window title for the given window ID.
    ///
    /// Detached tool windows get fixed titles ("Function Library — newc" etc.).
    /// The main window title includes the active project name when one is open.
    pub fn title_for_window(&self, window: iced::window::Id) -> String {
        if Some(window) == self.state.library_window { return "Function Library — newc".into(); }
        if Some(window) == self.state.cref_window    { return "C Reference — newc".into(); }
        if Some(window) == self.state.snippets_window { return "Snippets — newc".into(); }
        match &self.state.view {
            View::ProjectDetail(p) | View::ProjectStats(p) | View::ModuleDetail { project: p, .. } => {
                format!("newc — {}", p.name)
            }
            _ => String::from("newc"),
        }
    }

    /// Return the iced [`Theme`] to use for any window (all windows share the same theme).
    pub fn theme_for_window(&self, _window: iced::window::Id) -> Theme {
        theme_from_name(&self.state.active_theme)
    }

    /// Central message handler — the iced `update` callback.
    ///
    /// Drains any buffered build output before dispatching the message so that
    /// build state is always current. Most branches mutate [`AppState`] in
    /// place and return [`Task::none`]; branches that need async work (file
    /// dialogs, update checks, scrolling) return an appropriate [`Task`].
    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Drain any buffered build output first
        self.drain_build_output();
        self.drain_lsp_hover();

        match message {
            Message::Navigate(view) => {
                // Pre-compute for views that need it
                match &view {
                    View::HealthDashboard(project) => {
                        if !self.state.health_computed {
                            views::health::compute_health(&mut self.state, project);
                        }
                    }
                    View::MainBuilder(project) => {
                        self.state.main_builder =
                            newc_core::main_builder::MainBuilderState::load_from_main_c(&project.root);
                        self.state.composer_selected = None;
                        if self.state.create_author.is_empty() {
                            self.state.create_author = newc_core::scaffold::detect_author();
                        }
                    }
                    View::CallGraph(_) | View::DependencyGraph(_) | View::FlowChart(_) => {
                        self.state.graph_pan_x = 0.0;
                        self.state.graph_pan_y = 0.0;
                        self.state.graph_zoom = 1.0;
                        self.state.graph_selected = None;
                    }
                    View::MakefileEditor(project) => {
                        let file_name = project.build_system.build_file_name();
                        let raw = std::fs::read_to_string(project.root.join(file_name))
                            .unwrap_or_default();
                        self.state.makefile_content =
                            iced::widget::text_editor::Content::with_text(&raw);
                        self.state.makefile_dirty = false;
                    }
                    View::HeaderEditor { project, module_name } => {
                        if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                            let content = newc_core::header::read_ignore_block(&m.header)
                                .unwrap_or_default();
                            self.state.header_editor_state.te_content =
                                iced::widget::text_editor::Content::with_text(&content);
                            self.state.header_editor_state.content = content;
                            self.state.header_editor_state.dirty = false;
                        }
                    }
                    _ => {}
                }
                self.state.view = view;
            }

            Message::OpenProject(path) => {
                self.open_project(path);
            }

            Message::AddKnownProject(path) => {
                if !self.state.known_projects.contains(&path) {
                    self.state.known_projects.push(path);
                    save_known_projects(&self.state.known_projects);
                }
            }

            Message::BrowseForProject => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Open newc Project")
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    |p| p.map(Message::OpenProject).unwrap_or(Message::None),
                );
            }

            Message::RefreshProject => {
                if let Some(root) = self.state.current_project().map(|p| p.root.clone()) {
                    self.open_project(root);
                }
            }

            Message::BuildStart(target) => {
                if let Some(project) = self.state.current_project() {
                    let cwd = project.root.clone();
                    let build_system = project.build_system;
                    self.state.build_target_current = target.clone();
                    self.state.build_state = BuildState::Running;
                    self.state.build_lines.clear();
                    self.state.diagnostics.clear();
                    self.state.diag_nav_index = 0;
                    let extra_cflags = self.state.build_profile_active.as_ref()
                        .and_then(|name| self.state.project_config.as_ref()
                            .and_then(|pc| pc.build_profiles.iter().find(|p| &p.name == name))
                            .map(|p| p.cflags.clone()))
                        .unwrap_or_default();
                    self.runner.run(&target, cwd, build_system, &self.state.build_run_args, &extra_cflags);
                }
            }

            Message::BuildKill => {
                self.runner.kill();
            }

            Message::BuildLine(_) => {
                // Handled in drain_build_output; this Message variant exists for subscription use
            }

            Message::ToggleBuildPanel => {
                self.state.build_panel_open = !self.state.build_panel_open;
            }

            Message::BuildPanelClear => {
                self.state.build_lines.clear();
                self.state.diagnostics.clear();
                self.state.diag_nav_index = 0;
            }

            Message::BuildAutoScrollToggle => {
                self.state.build_auto_scroll = !self.state.build_auto_scroll;
            }

            Message::BuildArgsChanged(args) => {
                self.state.build_run_args = args;
            }

            Message::BuildProfileSelect(name) => {
                self.state.build_profile_active = name;
            }

            Message::DiagTabRaw(raw) => {
                self.state.diag_tab_raw = raw;
            }

            Message::CreateName(s) => self.state.create_name = s,
            Message::CreateAuthor(s) => self.state.create_author = s,
            Message::CreateLocation(s) => self.state.create_location = s,

            Message::CreateLocationBrowse => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Project Location")
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    |p| p.map(|path| Message::CreateLocation(
                        path.to_string_lossy().into_owned()
                    )).unwrap_or(Message::None),
                );
            }

            Message::CreateGitToggle(v) => self.state.create_git = v,
            Message::CreateUseCmakeToggle(v) => self.state.create_use_cmake = v,
            Message::CreateTemplate(i) => self.state.selected_template = Some(i),

            Message::CreateInclude(name, val) => match name.as_str() {
                "input" => self.state.create_include_input = val,
                "math" => self.state.create_include_math = val,
                "display" => self.state.create_include_display = val,
                "array" => self.state.create_include_array = val,
                "strings" => self.state.create_include_strings = val,
                "linked_list" => self.state.create_include_linked_list = val,
                "files" => self.state.create_include_files = val,
                "test_utils" => self.state.create_include_test_utils = val,
                "unity" => self.state.create_use_unity = val,
                _ => {}
            },

            Message::CreateLicense(id) => self.state.create_license = id,

            Message::CreateSubmit => {
                self.handle_create_project();
            }

            Message::AddModuleName(s) => self.state.add_module_name = s,

            Message::AddModuleSubmit => {
                if let View::AddModule { project } = self.state.view.clone() {
                    let name = self.state.add_module_name.trim().to_string();
                    if !name.is_empty() {
                        match module::add_module(&project.root, &name) {
                            Ok(_) => {
                                self.state.set_status(format!("Module '{}' added.", name));
                                self.state.add_module_name.clear();
                                self.open_project(project.root);
                            }
                            Err(e) => self.state.set_error(e.to_string()),
                        }
                    }
                }
            }

            Message::RemoveModule(name) => {
                if let Some(project) = self.state.current_project().cloned() {
                    self.state.confirm_remove_module = Some((project, name));
                }
            }

            Message::ConfirmRemoveModule => {
                if let Some((project, name)) = self.state.confirm_remove_module.take() {
                    match module::remove_module(&project.root, &name) {
                        Ok(_) => {
                            self.state.set_status(format!("Module '{}' removed.", name));
                            self.open_project(project.root);
                        }
                        Err(e) => self.state.set_error(e.to_string()),
                    }
                }
            }

            Message::CancelRemoveModule => {
                self.state.confirm_remove_module = None;
            }

            Message::QuickSearchToggle => {
                self.state.quick_search.open = !self.state.quick_search.open;
                if self.state.quick_search.open {
                    self.state.quick_search.query.clear();
                    self.state.quick_search.cursor = 0;
                }
            }

            Message::QuickSearchQuery(q) => {
                self.state.quick_search.query = q;
                self.state.quick_search.cursor = 0;
            }

            Message::QuickSearchCursor(i) => {
                self.state.quick_search.cursor = i;
            }

            Message::QuickSearchClose => {
                self.state.quick_search.open = false;
            }

            Message::QuickSearchSelect(_) => {}

            Message::QuickSearchSelectFunc { name, header } => {
                self.state.quick_search.open = false;
                self.state.cref_search = name;
                self.state.cref_selected_header = Some(header).filter(|h| !h.is_empty());
                self.state.view = View::CReference;
            }

            Message::SettingsSave => {
                self.state.config = self.state.config_draft.clone();
                self.state.active_theme = self.state.config.theme.clone();
                if let Err(e) = self.state.config.save() {
                    self.state.set_error(e.to_string());
                } else {
                    self.state.set_status("Settings saved.");
                    self.state.view = View::Home;
                }
            }

            Message::SettingsDiscard => {
                self.state.config_draft = self.state.config.clone();
                self.state.view = View::Home;
            }

            Message::SettingsDraftEditor(s) => self.state.config_draft.editor = s,
            Message::SettingsDraftTerminal(s) => self.state.config_draft.terminal = s,
            Message::SettingsDraftTheme(s) => self.state.config_draft.theme = s,
            Message::SettingsDraftClangStyle(s) => self.state.config_draft.clang_format_style = s,
            Message::SettingsDraftCodeFontSize(v) => self.state.config_draft.code_font_size = v,

            // Apply theme immediately without requiring settings save
            Message::ThemeSelect(name) => {
                self.state.active_theme = name.clone();
                self.state.config.theme = name.clone();
                self.state.config_draft.theme = name;
                let _ = self.state.config.save();
            }

            Message::ProjectConfigDraftEditor(s)     => self.state.project_config_draft.editor = Some(s),
            Message::ProjectConfigDraftTerminal(s)    => self.state.project_config_draft.terminal = Some(s),
            Message::ProjectConfigDraftClangStyle(s)  => self.state.project_config_draft.clang_format_style = Some(s),
            Message::ProfileNameInput(s) => self.state.profile_name_input = s,
            Message::ProfileCflagsInput(s) => self.state.profile_cflags_input = s,
            Message::ProfileAdd => {
                let name = self.state.profile_name_input.trim().to_string();
                let cflags = self.state.profile_cflags_input.trim().to_string();
                if !name.is_empty() {
                    self.state.project_config_draft.build_profiles.push(
                        newc_core::config::BuildProfile { name, cflags }
                    );
                    self.state.profile_name_input.clear();
                    self.state.profile_cflags_input.clear();
                }
            }
            Message::ProfileRemove(name) => {
                self.state.project_config_draft.build_profiles.retain(|p| p.name != name);
            }
            Message::ProjectConfigSave => {
                if let Some(project) = self.state.current_project() {
                    let root = project.root.clone();
                    match self.state.project_config_draft.save_to(&root) {
                        Ok(_) => {
                            self.state.project_config = Some(self.state.project_config_draft.clone());
                            self.state.push_toast(crate::state::Toast::success("Project settings saved.".to_string()));
                        }
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }
            Message::ProjectConfigClear => {
                if let Some(project) = self.state.current_project() {
                    let path = project.root.join(".newc_config.toml");
                    let _ = std::fs::remove_file(&path);
                    self.state.project_config = None;
                    self.state.project_config_draft = newc_core::config::ProjectConfig::default();
                    self.state.push_toast(crate::state::Toast::info("Project settings cleared.".to_string()));
                }
            }

            Message::LibrarySave(func) => {
                self.function_lib.upsert(func.clone());
                if let Err(e) = FunctionLibrary::save_user_function(&func) {
                    self.state.set_error(e.to_string());
                } else {
                    self.state.set_status(format!("Saved '{}'.", func.name));
                }
            }

            Message::LibraryDelete(name) => {
                self.function_lib.remove(&name);
                self.state.set_status(format!("Deleted '{name}'."));
            }

            Message::LibraryToggleStar(name) => {
                if let Some(f) = self.function_lib.get_mut(&name) {
                    f.starred = !f.starred;
                    let func = f.clone();
                    let _ = FunctionLibrary::save_user_function(&func);
                }
            }

            Message::NotesSave => {
                if let Some(project) = self.state.current_project() {
                    let _ = notes::save(&project.root, &self.state.notes_content.text());
                    self.state.notes_dirty = false;
                    self.state.set_status("Notes saved.");
                }
            }

            Message::NotesEdit(action) => {
                self.state.notes_content.perform(action);
                if let Some(project) = self.state.current_project() {
                    let _ = notes::save(&project.root, &self.state.notes_content.text());
                }
            }

            Message::MakefileSave => {
                if let Some(project) = self.state.current_project() {
                    let file_name = project.build_system.build_file_name();
                    let path = project.root.join(file_name);
                    if let Err(e) = std::fs::write(&path, self.state.makefile_content.text()) {
                        self.state.set_error(e.to_string());
                    } else {
                        self.state.makefile_dirty = false;
                        self.state.set_status(format!("{file_name} saved."));
                    }
                }
            }

            Message::MakefileEdit(action) => {
                self.state.makefile_content.perform(action);
                self.state.makefile_dirty = true;
            }

            Message::MakefileToggleFlag(flag) => {
                let text = self.state.makefile_content.text();
                let updated = toggle_makefile_flag(&text, &flag);
                self.state.makefile_content = iced::widget::text_editor::Content::with_text(&updated);
                self.state.makefile_dirty = true;
            }

            Message::OpenInEditor => {
                if let Some(project) = self.state.current_project() {
                    let root = project.root.clone();
                    match std::process::Command::new("xdg-open").arg(&root).spawn() {
                        Ok(_) => self.state.push_toast(crate::state::Toast::info("Opened in file manager.")),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(format!("xdg-open: {e}"))),
                    }
                }
            }

            Message::ExportZip => {
                if let Some(project) = self.state.current_project() {
                    let root = project.root.clone();
                    let name = project.name.clone();
                    let downloads = dirs::download_dir()
                        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                    match newc_core::export::export_zip(&root, &name, &downloads) {
                        Ok(path) => self.state.push_toast(crate::state::Toast::success(
                            format!("Exported → {}", path.display())
                        )),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::ExportCompileCommands => {
                if let Some(project) = self.state.current_project() {
                    match newc_core::export::write_compile_commands(&project.root) {
                        Ok(path) => self.state.push_toast(crate::state::Toast::success(
                            format!("Wrote {}", path.display())
                        )),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::SubmissionStudentInput(s) => self.state.submission_student_input = s,
            Message::SubmissionAssignmentInput(s) => self.state.submission_assignment_input = s,
            Message::PackSubmission => {
                if let Some(project) = self.state.current_project() {
                    let root = project.root.clone();
                    let name = project.name.clone();
                    let student = self.state.submission_student_input.trim().to_string();
                    let assignment = self.state.submission_assignment_input.trim().to_string();
                    if student.is_empty() || assignment.is_empty() {
                        self.state.push_toast(crate::state::Toast::error("Enter student name and assignment number.".to_string()));
                    } else {
                        let downloads = dirs::download_dir()
                            .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
                            .unwrap_or_else(|| std::path::PathBuf::from("."));
                        match newc_core::export::pack_submission(&root, &name, &student, &assignment, &downloads) {
                            Ok(path) => self.state.push_toast(crate::state::Toast::success(
                                format!("Packed → {}", path.display())
                            )),
                            Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                        }
                    }
                }
            }

            Message::GenerateReport => {
                if let Some(project) = self.state.current_project() {
                    let md = newc_core::report::generate(&project.root, &project.name);
                    let path = project.root.join(format!("{}_report.md", project.name));
                    match std::fs::write(&path, &md) {
                        Ok(_) => {
                            let editor = &self.state.config.editor;
                            let _ = std::process::Command::new(editor).arg(&path).spawn();
                            self.state.push_toast(crate::state::Toast::success(
                                format!("Report → {}", path.display())
                            ));
                        }
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::RunCheck => {
                if let Some(project) = self.state.current_project() {
                    match newc_core::analysis::check(&project.root) {
                        Ok(funcs) if funcs.is_empty() => {
                            self.state.push_toast(crate::state::Toast::success("No unreachable functions."));
                        }
                        Ok(funcs) => {
                            let names: Vec<&str> = funcs.iter().map(|f| f.name.as_str()).collect();
                            self.state.push_toast(crate::state::Toast::info(
                                format!("Unreachable: {}", names.join(", "))
                            ));
                        }
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::SyncAll => {
                if let Some(project) = self.state.current_project() {
                    match newc_core::sync::sync_all(&project.root) {
                        Ok(synced) if synced.is_empty() => {
                            self.state.push_toast(crate::state::Toast::info("Nothing to sync."));
                        }
                        Ok(synced) => {
                            self.state.push_toast(crate::state::Toast::success(
                                format!("Synced: {}", synced.join(", "))
                            ));
                        }
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::SyncModule(name) => {
                if let Some(project) = self.state.current_project() {
                    match newc_core::sync::sync_module(&project.root, &name) {
                        Ok(_) => self.state.push_toast(crate::state::Toast::success(format!("Synced {name}.h"))),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::FileChanged(path) => {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
                    .to_string();
                self.state.push_toast(crate::state::Toast::info(format!("{name} changed.")));
                // Warn if in module edit mode and the changed file is the open module
                if let View::ModuleDetail { module_name, .. } = &self.state.view.clone() {
                    if self.state.module_detail_state.edit_mode {
                        let stem_matches = path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s == module_name)
                            .unwrap_or(false);
                        if stem_matches {
                            self.state.push_toast(crate::state::Toast::error(
                                format!("{name} changed externally — unsaved edits may conflict.")
                            ));
                        }
                    }
                }
                return self.update(Message::RefreshProject);
            }

            Message::OnboardingToggleProject(i) => {
                if let Some(entry) = self.state.onboarding_found.get_mut(i) {
                    entry.1 = !entry.1;
                }
            }

            Message::OnboardingNext => {
                if let View::Onboarding(step) = self.state.view {
                    self.state.view = View::Onboarding(step + 1);
                }
            }

            Message::OnboardingBack => {
                if let View::Onboarding(step) = self.state.view {
                    if step > 0 {
                        self.state.view = View::Onboarding(step - 1);
                    }
                }
            }

            Message::OnboardingFinish => {
                // Import selected projects
                let selected: Vec<std::path::PathBuf> = self.state.onboarding_found.iter()
                    .filter(|(_, sel)| *sel)
                    .map(|(p, _)| p.clone())
                    .collect();
                for path in selected {
                    if !self.state.known_projects.contains(&path) {
                        self.state.known_projects.push(path);
                    }
                }
                crate::state::save_known_projects(&self.state.known_projects);
                // Save author name into config
                self.state.config.save().ok();
                // Write .onboarded marker
                if let Some(dir) = dirs::config_dir().map(|d| d.join("newc")) {
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = std::fs::write(dir.join(".onboarded"), "");
                }
                self.state.is_first_run = false;
                self.state.view = View::Home;
            }

            Message::ComposerDragStart(i) => {
                self.state.composer_drag = if self.state.composer_drag == Some(i) {
                    None // toggle off
                } else {
                    Some(i)
                };
            }

            Message::ComposerDragDrop(j) => {
                if let Some(i) = self.state.composer_drag.take() {
                    let len = self.state.main_builder.blocks.len();
                    if i != j && i < len && j <= len {
                        let snap = self.state.main_builder.clone();
                        let block = self.state.main_builder.blocks.remove(i);
                        let target = if j > i { j - 1 } else { j };
                        self.state.main_builder.blocks.insert(target, block);
                        self.state.composer_undo.push(snap);
                        self.state.composer_undo.truncate(50);
                        self.state.composer_redo.clear();
                    }
                }
            }

            Message::ComposerDragEnd => {
                self.state.composer_drag = None;
            }

            Message::GitCommitMsg(s) => self.state.git_commit_msg = s,
            Message::GitNewBranch(s) => self.state.git_new_branch = s,
            Message::GitShowDiff(v) => self.state.git_show_diff = v,
            Message::GitDiffStaged(v) => self.state.git_diff_staged = v,

            Message::GitCommit => {
                if let Some(project) = self.state.current_project() {
                    let root = project.root.clone();
                    let commit_message = self.state.git_commit_msg.trim().to_string();
                    if commit_message.is_empty() { return Task::none(); }
                    match newc_core::git::commit(&root, &commit_message) {
                        Ok(_) => {
                            self.state.git_commit_msg.clear();
                            self.state.set_status("Committed.");
                        }
                        Err(e) => self.state.set_error(e.to_string()),
                    }
                }
            }

            Message::GitPull => {
                if let Some(p) = self.state.current_project() {
                    match newc_core::git::pull(&p.root) {
                        Ok(_) => self.state.set_status("Pulled."),
                        Err(e) => self.state.set_error(e.to_string()),
                    }
                }
            }

            Message::GitPush => {
                if let Some(p) = self.state.current_project() {
                    match newc_core::git::push(&p.root) {
                        Ok(_) => self.state.set_status("Pushed."),
                        Err(e) => self.state.set_error(e.to_string()),
                    }
                }
            }

            Message::GitStash => {
                if let Some(p) = self.state.current_project() {
                    match newc_core::git::stash(&p.root) {
                        Ok(()) => self.state.set_status("Stashed."),
                        Err(e) => self.state.set_error(e.to_string()),
                    }
                }
            }

            Message::GitStashPop => {
                if let Some(p) = self.state.current_project() {
                    match newc_core::git::stash_pop(&p.root) {
                        Ok(()) => self.state.set_status("Stash popped."),
                        Err(e) => self.state.set_error(e.to_string()),
                    }
                }
            }

            Message::GitCreateBranch => {
                if let Some(p) = self.state.current_project() {
                    let name = self.state.git_new_branch.trim().to_string();
                    if !name.is_empty() {
                        match newc_core::git::new_branch(&p.root, &name) {
                            Ok(_) => {
                                self.state.git_new_branch.clear();
                                self.state.set_status(format!("Branch '{name}' created."));
                            }
                            Err(e) => self.state.set_error(e.to_string()),
                        }
                    }
                }
            }

            Message::GitCheckout(branch) => {
                if let Some(p) = self.state.current_project() {
                    let _ = newc_core::git::switch_branch(&p.root, &branch);
                }
            }

            Message::GitDeleteBranch(_branch) => {
                // delete_branch not available in newc_core::git — no-op until added
            }
            Message::GitInit => {
                if let Some(project) = self.state.current_project() {
                    match newc_core::git::init(&project.root) {
                        Ok(_) => self.state.push_toast(crate::state::Toast::success("Git repository initialized.".to_string())),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::SearchQuery(s) => self.state.search_query = s,
            Message::SearchSubmit => {
                if let Some(project) = self.state.current_project() {
                    let query = self.state.search_query.trim().to_string();
                    if !query.is_empty() {
                        self.state.search_results = newc_core::grep::search(&project.root, &query);
                    }
                }
            }

            Message::SearchReplaceInput(s) => self.state.search_replace = s,
            Message::SearchReplacePreview => {
                if let Some(project) = self.state.current_project() {
                    let query = self.state.search_query.trim().to_string();
                    let replacement = self.state.search_replace.clone();
                    if !query.is_empty() {
                        self.state.replace_preview = newc_core::grep::preview_replacements(&project.root, &query, &replacement);
                    }
                }
            }
            Message::SearchReplaceApply => {
                if let Some(project) = self.state.current_project().cloned() {
                    let query = self.state.search_query.trim().to_string();
                    let replacement = self.state.search_replace.clone();
                    if !query.is_empty() {
                        match newc_core::grep::apply_replacements(&project.root, &query, &replacement) {
                            Ok(n) => {
                                self.state.replace_preview.clear();
                                self.state.search_results = newc_core::grep::search(&project.root, &query);
                                self.state.push_toast(crate::state::Toast::success(format!("Replaced in {n} file(s).")));
                            }
                            Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                        }
                    }
                }
            }

            Message::UsageSearch(s) => self.state.usage_search = s,

            Message::ErrorDismiss => self.state.error_msg = None,

            Message::ShowShortcuts(v) => self.state.show_shortcuts = v,

            Message::ShowSaveTemplate(v) => self.state.show_save_template_modal = v,
            Message::SaveTemplateName(s) => self.state.save_template_name = s,
            Message::SaveTemplateDesc(s) => self.state.save_template_desc = s,
            Message::SaveTemplateSubmit => {
                if let Some(project) = self.state.current_project() {
                    let user_template = newc_core::user_template::UserTemplate {
                        name: self.state.save_template_name.trim().to_string(),
                        description: self.state.save_template_desc.trim().to_string(),
                        modules: project.modules.iter().map(|m| m.name.clone()).collect(),
                        blocks: Vec::new(),
                        globals: Vec::new(),
                    };
                    match newc_core::user_template::save(&user_template) {
                        Ok(_) => {
                            self.state.push_toast(crate::state::Toast::success(
                                format!("Template '{}' saved.", user_template.name)
                            ));
                            self.state.show_save_template_modal = false;
                            self.state.save_template_name.clear();
                            self.state.save_template_desc.clear();
                        }
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::ShowTidyConfirm(v) => self.state.show_tidy_confirm = v,
            Message::TidyConfirm => {
                if let Some(project) = self.state.current_project().cloned() {
                    let candidates = self.state.tidy_candidates.clone();
                    match newc_core::analysis::tidy(&project.root, &candidates) {
                        Ok(log) => {
                            self.state.push_toast(crate::state::Toast::success(
                                format!("Removed {} function(s).", log.len())
                            ));
                            self.state.tidy_candidates.clear();
                        }
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                    self.state.show_tidy_confirm = false;
                }
            }

            Message::WorkspaceSelect(ws) => self.state.active_workspace = ws,
            Message::WorkspaceInput(s) => self.state.workspace_input = s,
            Message::WorkspaceNew => {
                let name = self.state.workspace_input.trim().to_string();
                if !name.is_empty() {
                    self.state.config.workspaces.push(newc_core::config::Workspace {
                        name: name.clone(),
                        paths: Vec::new(),
                    });
                    let _ = self.state.config.save();
                    self.state.workspace_input.clear();
                    self.state.show_new_workspace = false;
                    self.state.active_workspace = Some(name);
                }
            }
            Message::WorkspaceCancelNew => {
                self.state.show_new_workspace = false;
                self.state.workspace_input.clear();
            }
            Message::ShowArchivedToggle => self.state.show_archived = !self.state.show_archived,

            Message::MoveToWorkspace(project_root) => {
                if let Some(target_ws) = &self.state.active_workspace.clone() {
                    // Remove from any workspace that contains this path
                    for ws in &mut self.state.config.workspaces {
                        ws.paths.retain(|p| p != &project_root);
                    }
                    // Add to target workspace
                    if let Some(ws) = self.state.config.workspaces.iter_mut().find(|w| &w.name == target_ws) {
                        ws.paths.push(project_root);
                    }
                    let _ = self.state.config.save();
                    self.state.push_toast(crate::state::Toast::success(
                        format!("Project moved to workspace '{target_ws}'.")
                    ));
                }
            }

            Message::MetaShowEditor(v) => self.state.show_meta_editor = v,
            Message::MetaCourse(s) => self.state.meta_draft.course = s,
            Message::MetaVersion(s) => self.state.meta_draft.assignment = s,
            Message::MetaSave => {
                if let Some(p) = self.state.current_project() {
                    let _ = meta::save(&p.root, &self.state.meta_draft);
                    self.state.show_meta_editor = false;
                    self.state.set_status("Metadata saved.");
                }
            }

            Message::ShowImport(v) => self.state.show_import = v,
            Message::ImportPickFile => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Import .c file")
                            .add_filter("C source", &["c"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    |p| {
                        if let Some(path) = p {
                            // Will be handled in import_c view port
                            Message::CreateLocation(path.to_string_lossy().into_owned())
                        } else {
                            Message::None
                        }
                    },
                );
            }

            Message::ShowNewGroup(v) => self.state.show_new_group = v,
            Message::NewGroupName(s) => self.state.new_group_name = s,
            Message::NewGroupDesc(s) => self.state.new_group_desc = s,
            Message::NewGroupSubmit => {
                let name = self.state.new_group_name.trim().to_string();
                if !name.is_empty() {
                    self.function_lib.create_group(&name, &self.state.new_group_desc);
                    self.state.show_new_group = false;
                    self.state.new_group_name.clear();
                    self.state.new_group_desc.clear();
                }
            }

            Message::GroupActionTarget(t) => self.state.group_action_target = t,
            Message::GroupRenameInput(s) => self.state.group_rename_input = s,
            Message::GroupRenameSubmit => {
                if let Some(old) = self.state.group_action_target.take() {
                    let new_name = self.state.group_rename_input.trim().to_string();
                    if !new_name.is_empty() {
                        self.function_lib.rename_group(&old, &new_name);
                    }
                }
            }
            Message::GroupDeleteCascade(v) => self.state.delete_group_cascade = v,
            Message::GroupDeleteSubmit => {
                if let Some(name) = self.state.group_action_target.take() {
                    self.function_lib.delete_group(&name, self.state.delete_group_cascade);
                }
            }

            Message::UpdateCheck => {
                return Task::perform(
                    async { crate::updater::check() },
                    |r| match r {
                        Ok(Some(v)) => Message::UpdateCheckResult(Some(v)),
                        Ok(None)    => Message::UpdateCheckResult(None),
                        Err(e)      => {
                            eprintln!("update check error: {e}");
                            Message::UpdateCheckResult(None)
                        }
                    },
                );
            }
            Message::UpdateCheckResult(v) => {
                match v {
                    Some(ver) => self.state.push_toast(crate::state::Toast::info(
                        format!("v{ver} available. Click Install to update.")
                    )),
                    None => self.state.push_toast(crate::state::Toast::success("Already up to date.".to_string())),
                }
            }
            Message::UpdateInstall(_) => {
                self.state.push_toast(crate::state::Toast::info("Downloading update…".to_string()));
                return Task::perform(
                    async { crate::updater::update() },
                    |r| match r {
                        Ok(_)  => Message::UpdateInstallDone,
                        Err(e) => {
                            eprintln!("update install error: {e}");
                            Message::UpdateInstallDone
                        }
                    },
                );
            }
            Message::UpdateInstallDone => {
                self.state.push_toast(crate::state::Toast::success("Updated. Restart newc to apply.".to_string()));
            }

            Message::DiagJumpTo { module, line } => {
                if let Some(project) = self.state.current_project().cloned() {
                    self.state.module_detail_state.highlight_line = Some(line);
                    self.state.view = View::ModuleDetail { project, module_name: module };
                    let y = ((line as f32 - 1.0 - 5.0).max(0.0)) * 16.0;
                    return iced::widget::operation::scroll_to::<Message>(
                        iced::widget::Id::new(crate::highlight::MODULE_CODE_SCROLL),
                        iced::widget::scrollable::AbsoluteOffset { x: 0.0, y },
                    );
                }
            }

            Message::SplitCompareSelect(name) => {
                self.state.split_compare_module = name;
            }

            Message::ModuleHoverRequest(module_name) => {
                if let Some(project) = self.state.current_project().cloned() {
                    if let Some(m) = project.modules.iter().find(|m| m.name == module_name) {
                        if self.lsp_client.is_none() {
                            self.lsp_client = crate::lsp::LspClient::spawn(&project.root);
                        }
                        let uri = format!("file://{}", m.source.display());
                        if let Some(client) = &self.lsp_client {
                            if self.lsp_opened_modules.insert(module_name.clone()) {
                                let text = std::fs::read_to_string(&m.source).unwrap_or_default();
                                client.did_open(uri.clone(), text);
                            }
                            let pos = self.state.module_detail_state.edit_content.cursor().position;
                            client.hover(uri, pos.line as u32, pos.column as u32);
                        } else {
                            self.state.push_toast(crate::state::Toast::error(
                                "clangd not found on PATH — hover unavailable.".to_string()
                            ));
                        }
                    }
                }
            }

            Message::DiagNavPrev => {
                if !self.state.diagnostics.is_empty() {
                    self.state.diag_nav_index = self.state.diag_nav_index
                        .checked_sub(1)
                        .unwrap_or(self.state.diagnostics.len() - 1);
                    let target_diag = self.state.diagnostics[self.state.diag_nav_index].clone();
                    let module = std::path::Path::new(&target_diag.file)
                        .file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or(target_diag.file);
                    return self.update(Message::DiagJumpTo { module, line: target_diag.line });
                }
            }

            Message::DiagNavNext => {
                if !self.state.diagnostics.is_empty() {
                    self.state.diag_nav_index = (self.state.diag_nav_index + 1) % self.state.diagnostics.len();
                    let target_diag = self.state.diagnostics[self.state.diag_nav_index].clone();
                    let module = std::path::Path::new(&target_diag.file)
                        .file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or(target_diag.file);
                    return self.update(Message::DiagJumpTo { module, line: target_diag.line });
                }
            }

            // Library state mutations
            Message::LibrarySearch(s) => self.state.library_state.search = s,
            Message::LibrarySelect(sel) => {
                self.state.library_state.selected = sel;
                self.state.library_state.edit_mode = false;
                self.state.library_state.draft = None;
            }
            Message::LibraryGroupSelect(g) => {
                self.state.library_state.active_group = g;
                self.state.library_state.selected = None;
            }
            Message::LibraryEditMode(v) => {
                self.state.library_state.edit_mode = v;
                if !v { self.state.library_state.draft = None; }
            }
            Message::LibraryAddingNew(v) => {
                self.state.library_state.adding_new = v;
                if v {
                    let mut draft = crate::views::library::LibraryState::new_draft();
                    if let Some(g) = &self.state.library_state.active_group.clone() {
                        draft.module = g.clone();
                    }
                    self.state.library_state.draft = Some(draft);
                    self.state.library_state.selected = None;
                    self.state.library_state.edit_mode = true;
                }
            }
            Message::LibraryGroupNew => {
                self.state.show_new_group = true;
            }
            Message::LibraryDraftField(field, value) => {
                if let Some(draft) = &mut self.state.library_state.draft {
                    use crate::state::LibraryField::*;
                    match field {
                        Name => draft.name = value,
                        Module => draft.module = value,
                        Description => draft.description = value,
                        Signature => draft.signature = value,
                        Header => draft.header_code = value,
                        Impl => draft.impl_code = value,
                        Tags => draft.tags = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                        Notes => draft.notes = value,
                    }
                }
            }
            Message::LibraryUpdateNotes { name, notes } => {
                if let Some(f) = self.function_lib.get_mut(&name) {
                    f.notes = notes.clone();
                    let func = f.clone();
                    let _ = FunctionLibrary::save_user_function(&func);
                }
            }

            // Snippets
            Message::SnippetsCat(i) => {
                self.state.snippets_cat = i;
                self.state.snippets_selected = None;
            }
            Message::SnippetsSelect(sel) => self.state.snippets_selected = sel,
            Message::SnippetsCopy(code) => {
                self.state.push_toast(crate::state::Toast::success("Copied to clipboard.".to_string()));
                return iced::clipboard::write(code);
            }

            // Pane resize
            Message::LibraryPaneResized(e) => { self.state.library_panes.resize(e.split, e.ratio); }
            Message::CRefPaneResized(e) => { self.state.cref_panes.resize(e.split, e.ratio); }
            Message::SnippetsPaneResized(e) => { self.state.snippets_panes.resize(e.split, e.ratio); }
            Message::ModulePaneResized(e) => { self.state.module_panes.resize(e.split, e.ratio); }

            // Main builder
            Message::ComposerAddBlock(block) => {
                let snap = self.state.main_builder.clone();
                self.state.main_builder.blocks.push(block);
                self.state.composer_undo.push(snap);
                self.state.composer_undo.truncate(50);
                self.state.composer_redo.clear();
            }
            Message::ComposerUndo => {
                if let Some(prev) = self.state.composer_undo.pop() {
                    let cur = std::mem::replace(&mut self.state.main_builder, prev);
                    self.state.composer_redo.push(cur);
                    self.state.composer_redo.truncate(50);
                }
            }
            Message::ComposerRedo => {
                if let Some(next) = self.state.composer_redo.pop() {
                    let cur = std::mem::replace(&mut self.state.main_builder, next);
                    self.state.composer_undo.push(cur);
                    self.state.composer_undo.truncate(50);
                }
            }
            Message::ComposerBlockMoveUp(i) => {
                if i > 0 && i < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    self.state.main_builder.blocks.swap(i, i - 1);
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerBlockMoveDown(i) => {
                let len = self.state.main_builder.blocks.len();
                if i + 1 < len {
                    let snap = self.state.main_builder.clone();
                    self.state.main_builder.blocks.swap(i, i + 1);
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerBlockDelete(i) => {
                if i < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    self.state.main_builder.blocks.remove(i);
                    if self.state.composer_selected == Some(i) {
                        self.state.composer_selected = None;
                    }
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerBlockDuplicate(i) => {
                if i < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    let cloned = self.state.main_builder.blocks[i].clone();
                    self.state.main_builder.blocks.insert(i + 1, cloned);
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerAddChildBlock { parent, block } => {
                use newc_core::main_builder::MainBlock;
                if parent < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    match &mut self.state.main_builder.blocks[parent] {
                        MainBlock::IfBlock { body, .. }
                        | MainBlock::WhileLoop { body, .. }
                        | MainBlock::ForLoop { body, .. } => { body.push(block); }
                        _ => {}
                    }
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerDeleteChildBlock { parent, child } => {
                use newc_core::main_builder::MainBlock;
                if parent < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    match &mut self.state.main_builder.blocks[parent] {
                        MainBlock::IfBlock { body, .. }
                        | MainBlock::WhileLoop { body, .. }
                        | MainBlock::ForLoop { body, .. } => {
                            if child < body.len() { body.remove(child); }
                        }
                        _ => {}
                    }
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerMoveChildUp { parent, child } => {
                use newc_core::main_builder::MainBlock;
                if parent < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    match &mut self.state.main_builder.blocks[parent] {
                        MainBlock::IfBlock { body, .. }
                        | MainBlock::WhileLoop { body, .. }
                        | MainBlock::ForLoop { body, .. } => {
                            if child > 0 && child < body.len() { body.swap(child, child - 1); }
                        }
                        _ => {}
                    }
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerMoveChildDown { parent, child } => {
                use newc_core::main_builder::MainBlock;
                if parent < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    match &mut self.state.main_builder.blocks[parent] {
                        MainBlock::IfBlock { body, .. }
                        | MainBlock::WhileLoop { body, .. }
                        | MainBlock::ForLoop { body, .. } => {
                            if child + 1 < body.len() { body.swap(child, child + 1); }
                        }
                        _ => {}
                    }
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerToggleInclude(name) => {
                if let Some(pos) = self.state.main_builder.includes.iter().position(|i| i == &name) {
                    self.state.main_builder.includes.remove(pos);
                } else {
                    self.state.main_builder.includes.push(name);
                }
            }
            Message::ComposerSelectBlock(block_index) => {
                self.state.composer_selected = if self.state.composer_selected == Some(block_index) {
                    None
                } else {
                    Some(block_index)
                };
            }
            Message::ComposerEditField { block_index, field, value } => {
                use newc_core::main_builder::MainBlock;
                if block_index < self.state.main_builder.blocks.len() {
                    let snap = self.state.main_builder.clone();
                    match &mut self.state.main_builder.blocks[block_index] {
                        MainBlock::VarDecl { type_name, name, init, .. } => match field.as_str() {
                            "type" => *type_name = value,
                            "name" => *name = value,
                            "init" => *init = value,
                            _ => {}
                        },
                        MainBlock::FunctionCall { func_name, args, assign_to, comment } => {
                            match field.as_str() {
                                "func_name" => *func_name = value,
                                "args" => {
                                    *args = if value.trim().is_empty() {
                                        Vec::new()
                                    } else {
                                        value.split(',').map(|s| s.trim().to_string()).collect()
                                    };
                                }
                                "assign_to" => *assign_to = value,
                                "comment" => *comment = value,
                                _ => {}
                            }
                        }
                        MainBlock::IfBlock { condition, .. } => {
                            if field == "condition" { *condition = value; }
                        }
                        MainBlock::WhileLoop { condition, .. } => {
                            if field == "condition" { *condition = value; }
                        }
                        MainBlock::ForLoop { init, condition, increment, .. } => {
                            match field.as_str() {
                                "init" => *init = value,
                                "condition" => *condition = value,
                                "increment" => *increment = value,
                                _ => {}
                            }
                        }
                        MainBlock::Comment(c) => *c = value,
                        MainBlock::RawCode(c) => *c = value,
                        MainBlock::BlankLine => {}
                    }
                    self.state.composer_undo.push(snap);
                    self.state.composer_undo.truncate(50);
                    self.state.composer_redo.clear();
                }
            }
            Message::ComposerWriteMainC => {
                if let Some(project) = self.state.current_project().cloned() {
                    let author = self.state.create_author.clone();
                    let date = chrono::Local::now().format("%d/%m/%Y").to_string();
                    let code = self.state.main_builder.preview(&author, &date);
                    let main_c = project.root.join("src").join("main.c");
                    match std::fs::write(&main_c, &code) {
                        Ok(_) => self.state.set_status("main.c written."),
                        Err(e) => self.state.set_error(e.to_string()),
                    }
                }
            }

            // Module detail
            Message::ModuleSelectFunc(sel) => {
                self.state.module_detail_state.selected_func = sel;
                self.state.module_detail_state.edit_mode = false;
            }
            Message::ModuleEditMode(v) => {
                if v {
                    // load current function body into edit buffer
                    if let Some(selected_function_name) = &self.state.module_detail_state.selected_func.clone() {
                        if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                            if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                                if let Ok(src) = std::fs::read_to_string(&m.source) {
                                    let funcs = newc_core::sync::extract_function_implementations(&src);
                                    if let Some(f) = funcs.iter().find(|f| &f.name == selected_function_name) {
                                        self.state.module_detail_state.edit_content = iced::widget::text_editor::Content::with_text(&f.body);
                                    }
                                }
                            }
                        }
                    }
                }
                self.state.module_detail_state.edit_mode = v;
            }
            Message::ModuleSaveFunc { name, new_impl } => {
                if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                    if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                        let _ = newc_core::sync::update_function_in_source(&m.source, &name, &new_impl);
                        self.state.module_detail_state.edit_mode = false;
                        self.state.set_status(format!("Saved '{name}'."));
                    }
                }
            }
            Message::ModuleDeleteFunc(name) => {
                if name.is_empty() {
                    self.state.module_detail_state.delete_func_confirm = None;
                } else {
                    self.state.module_detail_state.delete_func_confirm = Some(name);
                }
            }
            Message::ModuleDeleteFuncConfirm => {
                if let Some(function_to_delete) = self.state.module_detail_state.delete_func_confirm.take() {
                    if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                        if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                            match newc_core::analysis::remove_function_from_source_pub(&m.source, &function_to_delete) {
                                Ok(_) => {
                                    let _ = newc_core::sync::sync_module(&project.root, module_name);
                                    self.state.module_detail_state.selected_func = None;
                                    self.state.push_toast(crate::state::Toast::success(
                                        format!("Deleted '{function_to_delete}'.")
                                    ));
                                }
                                Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                            }
                        }
                    }
                }
            }
            Message::ModuleRunCheck => {
                if let View::ModuleDetail { project, module_name: _ } = &self.state.view.clone() {
                    let funcs = newc_core::analysis::check(&project.root).unwrap_or_default();
                    self.state.module_detail_state.unreachable_funcs = funcs.into_iter().map(|f| f.name).collect();
                    self.state.module_detail_state.check_ran = true;
                }
            }
            Message::ModuleShowCallTree(v) => self.state.module_detail_state.show_call_tree = v,
            Message::ModuleEditAction(action) => {
                self.state.module_detail_state.edit_content.perform(action);
            }
            Message::ModuleAddFromLibrary => {
                if let View::ModuleDetail { module_name, .. } = &self.state.view.clone() {
                    self.state.pending_library_insert_module = Some(module_name.clone());
                    self.state.view = View::FunctionLibrary;
                }
            }
            Message::LibraryInsertToModule => {
                if let (Some(module_name), Some(sel)) = (
                    self.state.pending_library_insert_module.take(),
                    self.state.library_state.selected.clone(),
                ) {
                    let func = self.function_lib.all().iter().find(|f| f.name == sel).cloned();
                    if let Some(f) = func {
                        if let Some(project) = self.state.current_project().cloned() {
                            if let Some(m) = project.modules.iter().find(|m| m.name == module_name) {
                                let mut src = std::fs::read_to_string(&m.source).unwrap_or_default();
                                if !src.ends_with('\n') { src.push('\n'); }
                                src.push('\n');
                                src.push_str(f.impl_code.trim_end());
                                src.push('\n');
                                match std::fs::write(&m.source, &src) {
                                    Ok(_) => {
                                        let _ = newc_core::sync::sync_module(&project.root, &module_name);
                                        self.state.push_toast(crate::state::Toast::success(
                                            format!("Inserted '{}' into {module_name}.", f.name)
                                        ));
                                        self.state.view = View::ModuleDetail {
                                            project,
                                            module_name,
                                        };
                                    }
                                    Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                                }
                            }
                        }
                    }
                }
            }
            Message::ModuleSyncNow => {
                if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                    let _ = newc_core::sync::sync_module(&project.root, module_name);
                    self.state.set_status(format!("Synced {module_name}.h"));
                }
            }
            Message::LaunchDebugger => {
                if let Some(project) = self.state.current_project() {
                    let root = project.root.clone();
                    let binary = if root.join(&project.name).exists() {
                        format!("./{}", project.name)
                    } else {
                        "./main".to_string()
                    };
                    let terminal = self.state.config.terminal.clone();
                    let gdb_cmd = format!("gdb {binary}");
                    let result = match terminal.as_str() {
                        "alacritty" | "xterm" | "urxvt" => std::process::Command::new(&terminal)
                            .args(["-e", "gdb", &binary])
                            .current_dir(&root).spawn(),
                        "gnome-terminal" => std::process::Command::new(&terminal)
                            .args(["--", "gdb", &binary])
                            .current_dir(&root).spawn(),
                        "wezterm" => std::process::Command::new(&terminal)
                            .args(["start", "--", "gdb", &binary])
                            .current_dir(&root).spawn(),
                        _ => std::process::Command::new(&terminal)
                            .args(["gdb", &binary])
                            .current_dir(&root).spawn(),
                    };
                    match result {
                        Ok(_) => self.state.push_toast(crate::state::Toast::info(format!("Launched: {terminal} {gdb_cmd}"))),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(format!("Failed to launch debugger: {e}"))),
                    }
                }
            }

            Message::ModuleRenameStart(name) => {
                if name.is_empty() {
                    self.state.show_rename_modal = false;
                } else {
                    self.state.rename_func_old = name.clone();
                    self.state.rename_func_input = name;
                    self.state.show_rename_modal = true;
                }
            }
            Message::ModuleRenameInput(s) => self.state.rename_func_input = s,
            Message::ModuleRenameSubmit => {
                self.state.show_rename_modal = false;
                let old = self.state.rename_func_old.clone();
                let new = self.state.rename_func_input.trim().to_string();
                if !new.is_empty() && old != new {
                    if let Some(project) = self.state.current_project().cloned() {
                        match newc_core::refactor::rename_function(&project.root, &old, &new) {
                            Ok(n) => {
                                if let View::ModuleDetail { module_name, .. } = &self.state.view.clone() {
                                    let _ = newc_core::sync::sync_module(&project.root, module_name);
                                }
                                self.state.module_detail_state.selected_func = Some(new.clone());
                                self.state.push_toast(crate::state::Toast::success(
                                    format!("Renamed '{old}' → '{new}' in {n} file(s).")
                                ));
                            }
                            Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                        }
                    }
                }
            }

            Message::ModuleMoveStart(name) => {
                if name.is_empty() {
                    self.state.show_move_modal = false;
                } else {
                    self.state.move_func_name = name;
                    self.state.move_func_target_input = String::new();
                    self.state.show_move_modal = true;
                }
            }
            Message::ModuleMoveTargetInput(s) => self.state.move_func_target_input = s,
            Message::ModuleMoveSubmit => {
                self.state.show_move_modal = false;
                let move_function_name = self.state.move_func_name.clone();
                let target = self.state.move_func_target_input.trim().to_string();
                if !move_function_name.is_empty() && !target.is_empty() {
                    if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                        if target != *module_name {
                            match newc_core::refactor::move_function(&project.root, module_name, &target, &move_function_name) {
                                Ok(_) => {
                                    self.state.module_detail_state.selected_func = None;
                                    self.state.push_toast(crate::state::Toast::success(
                                        format!("Moved '{move_function_name}' → {target}.")
                                    ));
                                }
                                Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                            }
                        }
                    }
                }
            }

            Message::ModuleGenerateDoc(function_name) => {
                if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                    match newc_core::doc::insert_stub(&project.root, module_name, &function_name) {
                        Ok(()) => self.state.push_toast(crate::state::Toast::success(
                            format!("Inserted doc stub for '{function_name}'.")
                        )),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::LintQuickFix { module, function, line_no, code } => {
                if let Some(project) = self.state.current_project() {
                    match newc_core::lint::apply_fix_in_function(&project.root, &module, &function, line_no, &code) {
                        Ok(()) => self.state.push_toast(crate::state::Toast::success(
                            format!("Applied {code} fix.")
                        )),
                        Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                    }
                }
            }

            Message::ModuleClangFormat => {
                if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                    if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                        let style = self.state.config.clang_format_style.clone();
                        let result = std::process::Command::new("clang-format")
                            .arg(format!("--style={style}"))
                            .arg("-i")
                            .arg(&m.source)
                            .status();
                        match result {
                            Ok(s) if s.success() => self.state.push_toast(crate::state::Toast::success("clang-format applied.".to_string())),
                            Ok(_) => self.state.push_toast(crate::state::Toast::error("clang-format returned non-zero.".to_string())),
                            Err(e) => self.state.push_toast(crate::state::Toast::error(format!("clang-format: {e}"))),
                        }
                    }
                }
            }

            // Header
            Message::HeaderSave => {
                if let View::HeaderEditor { project, module_name } = &self.state.view.clone() {
                    if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                        let content = self.state.header_editor_state.te_content.text();
                        let _ = newc_core::header::write_ignore_block(&m.header, &content);
                        self.state.header_editor_state.dirty = false;
                        self.state.set_status("Header saved.");
                    }
                }
            }
            Message::HeaderEditorAction(action) => {
                self.state.header_editor_state.te_content.perform(action);
                self.state.header_editor_state.dirty = true;
            }
            Message::HeaderContent(s) => {
                self.state.header_editor_state.content = s;
                self.state.header_editor_state.dirty = true;
            }

            // Import
            Message::ImportTargetModule(s) => self.state.import_state.target_module = s,
            Message::ImportToggleFunc(i) => {
                if i == usize::MAX {
                    self.state.import_state.selected.iter_mut().for_each(|s| *s = true);
                } else if i == usize::MAX - 1 {
                    self.state.import_state.selected.iter_mut().for_each(|s| *s = false);
                } else if let Some(s) = self.state.import_state.selected.get_mut(i) {
                    *s = !*s;
                }
            }
            Message::ImportSubmit => {
                let funcs = crate::views::import_c::build_templates(&self.state.import_state);
                for func in funcs {
                    self.function_lib.upsert(func.clone());
                    let _ = FunctionLibrary::save_user_function(&func);
                }
                self.state.show_import = false;
                self.state.set_status("Functions imported.");
            }
            Message::ImportExtracted(s) => self.state.import_state = s,

            // CRef
            Message::CRefSearch(s) => {
                self.state.cref_search = s;
                self.state.cref_selected_func = None;
            }
            Message::CRefSelectHeader(h) => {
                self.state.cref_selected_header = h;
                self.state.cref_search.clear();
                self.state.cref_selected_func = None;
            }
            Message::CRefSelectFunc(f) => self.state.cref_selected_func = f,

            // Snippets state (already handled above)

            // Git extras
            Message::GitStage(path) => {
                if let Some(p) = self.state.current_project() {
                    let _ = newc_core::git::stage_file(&p.root.clone(), &path);
                }
            }
            Message::GitUnstage(path) => {
                if let Some(p) = self.state.current_project() {
                    let _ = newc_core::git::unstage_file(&p.root.clone(), &path);
                }
            }

            // Subscription ticks
            Message::PollBuildOutput => {
                // drain_build_output() already called at top of update()
            }
            Message::StatusTick => {
                if let Some((_, t)) = &self.state.status {
                    if t.elapsed().as_secs() >= 4 {
                        self.state.status = None;
                    }
                }
            }
            Message::ToastTick => {
                for toast in &mut self.state.toasts {
                    toast.elapsed_ms = toast.elapsed_ms.saturating_add(100);
                }
                self.state.toasts.retain(|t| !t.is_expired());
            }
            Message::ToastDismiss(i) => {
                if i < self.state.toasts.len() {
                    self.state.toasts.remove(i);
                }
            }

            // Graph canvas
            Message::GraphNodeSelect(name) => self.state.graph_selected = Some(name),
            Message::GraphPan { dx, dy } => {
                self.state.graph_pan_x += dx;
                self.state.graph_pan_y += dy;
            }
            Message::GraphZoom(delta) => {
                self.state.graph_zoom = (self.state.graph_zoom + delta).clamp(0.2, 4.0);
            }
            Message::GraphReset => {
                self.state.graph_pan_x = 0.0;
                self.state.graph_pan_y = 0.0;
                self.state.graph_zoom = 1.0;
                self.state.graph_selected = None;
            }
            Message::GraphExport => {
                let result = match &self.state.view {
                    View::CallGraph(p) => {
                        let svg = views::call_graph::export_svg(p);
                        let path = p.root.join(format!("{}_call_graph.svg", p.name));
                        std::fs::write(&path, svg).map(|_| path)
                    }
                    View::DependencyGraph(p) => {
                        let svg = views::dependency_graph::export_svg(p);
                        let path = p.root.join(format!("{}_dep_graph.svg", p.name));
                        std::fs::write(&path, svg).map(|_| path)
                    }
                    _ => {
                        self.state.push_toast(crate::state::Toast::info("Export only available in graph views."));
                        return Task::none();
                    }
                };
                match result {
                    Ok(path) => self.state.push_toast(crate::state::Toast::success(
                        format!("SVG → {}", path.display())
                    )),
                    Err(e) => self.state.push_toast(crate::state::Toast::error(e.to_string())),
                }
            }

            // Multi-window: Library / CRef / Snippets
            Message::OpenLibraryWindow => {
                if let Some(id) = self.state.library_window {
                    return iced::window::gain_focus(id);
                }
                let (id, task) = iced::window::open(iced::window::Settings {
                    size: iced::Size::new(720.0, 880.0),
                    min_size: Some(iced::Size::new(500.0, 400.0)),
                    position: iced::window::Position::Default,
                    ..Default::default()
                });
                self.state.library_window = Some(id);
                return task.discard();
            }
            Message::OpenCRefWindow => {
                if let Some(id) = self.state.cref_window {
                    return iced::window::gain_focus(id);
                }
                let (id, task) = iced::window::open(iced::window::Settings {
                    size: iced::Size::new(700.0, 860.0),
                    min_size: Some(iced::Size::new(500.0, 400.0)),
                    position: iced::window::Position::Default,
                    ..Default::default()
                });
                self.state.cref_window = Some(id);
                return task.discard();
            }
            Message::OpenSnippetsWindow => {
                if let Some(id) = self.state.snippets_window {
                    return iced::window::gain_focus(id);
                }
                let (id, task) = iced::window::open(iced::window::Settings {
                    size: iced::Size::new(700.0, 860.0),
                    min_size: Some(iced::Size::new(500.0, 400.0)),
                    position: iced::window::Position::Default,
                    ..Default::default()
                });
                self.state.snippets_window = Some(id);
                return task.discard();
            }
            Message::WindowClosed(id) => {
                if self.state.library_window == Some(id) { self.state.library_window = None; }
                if self.state.cref_window    == Some(id) { self.state.cref_window    = None; }
                if self.state.snippets_window== Some(id) { self.state.snippets_window= None; }
            }

            Message::None => {}
            _ => {}
        }

        Task::none()
    }

    /// Render the UI for the given window — the iced `view` callback.
    ///
    /// Detached tool windows (Library, CRef, Snippets) are rendered by their
    /// dedicated view functions. All other windows fall through to
    /// [`Self::view_main`] which composes the top bar, sidebar, central panel,
    /// build panel, and toast overlay.
    pub fn view_for_window(&self, window: iced::window::Id) -> Element<'_, Message> {
        if Some(window) == self.state.library_window {
            return container(views::library::view(&self.state, &self.function_lib))
                .width(Length::Fill).height(Length::Fill).style(th::panel_style).into();
        }
        if Some(window) == self.state.cref_window {
            return container(views::cref::view(&self.state))
                .width(Length::Fill).height(Length::Fill).style(th::panel_style).into();
        }
        if Some(window) == self.state.snippets_window {
            return container(views::snippets::view(&self.state))
                .width(Length::Fill).height(Length::Fill).style(th::panel_style).into();
        }
        self.view_main()
    }

    fn view_main(&self) -> Element<'_, Message> {
        let top_bar = self.top_bar();
        let sidebar = self.sidebar();
        let mut central = self.central_panel();

        // Overlay: error modal (shown on top of central content)
        if let Some(err) = &self.state.error_msg {
            let err_clone = err.clone();
            central = container(
                column![
                    text("Error").size(16).color(Color::from_rgb(1.0, 0.376, 0.533)),
                    text(err_clone),
                    button(text("Dismiss")).on_press(Message::ErrorDismiss),
                ]
                .spacing(8)
                .padding(20),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        // Overlay: shortcuts (Stack over existing content with semi-transparent backdrop)
        if self.state.show_shortcuts {
            let backdrop = container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.6).into()),
                    ..Default::default()
                });
            let panel = container(views::shortcuts::view(&self.state))
                .max_width(520)
                .width(Length::Shrink)
                .center(Length::Fill);
            central = Stack::new()
                .push(central)
                .push(backdrop)
                .push(panel)
                .into();
        }

        // Overlay: quick search (Stack over existing content with semi-transparent backdrop)
        if self.state.quick_search.open {
            let backdrop = container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.6).into()),
                    ..Default::default()
                });
            let panel = container(views::quick_search::view(&self.state))
                .max_width(560)
                .width(Length::Shrink)
                .center(Length::Fill);
            central = Stack::new()
                .push(central)
                .push(backdrop)
                .push(panel)
                .into();
        }

        let content = row![sidebar, central]
            .width(Length::Fill)
            .height(Length::Fill);

        let status_bar = self.status_bar();
        let mut items: Vec<Element<Message>> = vec![top_bar, content.into(), status_bar];
        if self.state.build_panel_open {
            items.push(views::build_panel::view(&self.state));
        }

        let main_col: Element<Message> = iced::widget::Column::with_children(items)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();

        // Toast overlay — stack above the main layout, bottom-right aligned
        if self.state.toasts.is_empty() {
            return main_col;
        }

        let toast_widgets: Vec<Element<Message>> = self.state.toasts.iter().enumerate()
            .rev() // newest on bottom
            .map(|(i, t)| {
                container(
                    row![
                        text(t.message.clone()).size(12).color(th::color::TEXT),
                        Space::new().width(Length::Fill),
                        button(text("×").size(10).color(th::color::TEXT_DIM))
                            .style(th::btn_ghost)
                            .on_press(Message::ToastDismiss(i)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                )
                .style(|_| th::toast_style(&t.kind))
                .padding([8, 12])
                .max_width(300)
                .into()
            })
            .collect();

        let toast_overlay: Element<Message> = container(
            column(toast_widgets).spacing(6)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(iced::Padding { top: 0.0, right: 16.0, bottom: 52.0, left: 0.0 })
        .into();

        Stack::new()
            .push(main_col)
            .push(toast_overlay)
            .into()
    }

    /// Build the active [`Subscription`] set.
    ///
    /// Subscriptions are activated only when needed:
    /// - 50 ms poll timer while a build is [`BuildState::Running`]
    /// - 1 s status-clear timer while a status message is visible
    /// - 100 ms toast tick timer while toasts are present
    /// - File-system watcher on the current project root
    /// - Window close events (to clean up detached window IDs)
    /// - Global keyboard shortcuts (Ctrl+B build, Ctrl+R refresh, Ctrl+P quick search, etc.)
    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs: Vec<Subscription<Message>> = Vec::new();

        // Live build output: poll runner every 50ms while building
        if matches!(self.state.build_state, BuildState::Running) {
            subs.push(
                iced::time::every(Duration::from_millis(50))
                    .map(|_| Message::PollBuildOutput),
            );
        }

        // Status auto-clear: tick every second
        if self.state.status.is_some() {
            subs.push(
                iced::time::every(Duration::from_secs(1))
                    .map(|_| Message::StatusTick),
            );
        }

        // Toast auto-dismiss: tick every 100ms when toasts active
        if !self.state.toasts.is_empty() {
            subs.push(
                iced::time::every(Duration::from_millis(100))
                    .map(|_| Message::ToastTick),
            );
        }

        // File watcher — watch current project root for external changes
        if let Some(root) = self.state.current_project().map(|p| p.root.clone()) {
            subs.push(Subscription::run_with(root, file_watch_stream));
        }

        // Window close events — clear window ID from state
        subs.push(iced::window::close_events().map(Message::WindowClosed));

        // Global keyboard shortcuts
        subs.push(iced::event::listen_with(|event, _status, _window| {
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                match (modifiers.control(), modifiers.shift(), key.as_ref()) {
                    (true, _, iced::keyboard::Key::Character("b")) => Some(Message::BuildStart("all".into())),
                    (true, _, iced::keyboard::Key::Character("r")) => Some(Message::RefreshProject),
                    (true, true, iced::keyboard::Key::Character("z")) => Some(Message::ComposerRedo),
                    (true, false, iced::keyboard::Key::Character("z")) => Some(Message::ComposerUndo),
                    (true, _, iced::keyboard::Key::Character("y")) => Some(Message::ComposerRedo),
                    (true, false, iced::keyboard::Key::Character("p")) => Some(Message::QuickSearchToggle),
                    _ => {
                        if let iced::keyboard::Key::Character(c) = key.as_ref() {
                            if c == "?" { return Some(Message::ShowShortcuts(true)); }
                        }
                        None
                    }
                }
            } else {
                None
            }
        }));

        if subs.is_empty() {
            Subscription::none()
        } else {
            Subscription::batch(subs)
        }
    }
}

// ── Private helpers ────────────────────────────────────────────────────────────

impl NewcApp {
    fn drain_lsp_hover(&mut self) {
        if let Some(client) = &self.lsp_client {
            if let Some(result) = client.try_recv_hover() {
                self.state.module_detail_state.hover_text = Some(result.text);
            }
        }
    }

    fn drain_build_output(&mut self) {
        let mut new_stderr_lines: Vec<String> = Vec::new();
        for line in self.runner.drain() {
            if let LineKind::Done { exit_code, duration_ms } = line.kind {
                self.state.build_state = BuildState::Done { exit_code };
                if let Some(project) = self.state.current_project() {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    build_history::append(
                        &project.root.clone(),
                        BuildRecord {
                            timestamp,
                            target: self.state.build_target_current.clone(),
                            exit_code,
                            duration_ms,
                        },
                    );
                }
                self.state.diagnostics = diag::parse(&new_stderr_lines);
                self.state.diag_nav_index = 0;
                if self.state.build_target_current == "valgrind-xml" {
                    if let Some(project) = self.state.current_project() {
                        let xml = std::fs::read_to_string(project.root.join("vg.xml")).unwrap_or_default();
                        self.state.valgrind_errors = newc_core::valgrind::parse(&xml);
                    }
                }
                self.state.build_lines.push(line);
            } else {
                if matches!(line.kind, LineKind::Stderr) {
                    new_stderr_lines.push(line.text.clone());
                }
                self.state.build_state = BuildState::Running;
                self.state.build_lines.push(line);
            }
        }
    }

    fn open_project(&mut self, path: PathBuf) {
        match Project::open(path.clone()) {
            Ok(p) => {
                if !self.state.known_projects.contains(&path) {
                    self.state.known_projects.push(path.clone());
                    save_known_projects(&self.state.known_projects);
                }
                self.state.recent_projects.retain(|r| r != &path);
                self.state.recent_projects.insert(0, path.clone());
                self.state.recent_projects.truncate(5);
                crate::state::save_recent_projects(&self.state.recent_projects);

                self.state.cached_stats = None;
                self.state.health_computed = false;
                self.state.meta_draft = meta::load(&p.root);
                self.state.notes_content =
                    iced::widget::text_editor::Content::with_text(&notes::load(&p.root));
                self.state.notes_dirty = false;
                // Load per-project config overrides
                self.state.project_config = newc_core::config::ProjectConfig::load_from(&p.root);
                self.state.project_config_draft = self.state.project_config.clone().unwrap_or_default();
                self.state.view = View::ProjectDetail(p);
            }
            Err(e) => self.state.set_error(e.to_string()),
        }
    }

    fn handle_create_project(&mut self) {
        let name = self.state.create_name.trim().to_string();
        if name.is_empty() {
            self.state.set_error("Project name cannot be empty.");
            return;
        }

        let parent = std::path::PathBuf::from(&self.state.create_location);
        let root = parent.join(&name);

        let mut modules = Vec::new();
        if self.state.create_include_input { modules.push(DefaultModule::Input); }
        if self.state.create_include_math { modules.push(DefaultModule::Math); }
        if self.state.create_include_display { modules.push(DefaultModule::Display); }
        if self.state.create_include_array { modules.push(DefaultModule::Array); }
        if self.state.create_include_strings { modules.push(DefaultModule::Strings); }
        if self.state.create_include_linked_list { modules.push(DefaultModule::LinkedList); }
        if self.state.create_include_files { modules.push(DefaultModule::Files); }
        if self.state.create_include_test_utils { modules.push(DefaultModule::TestUtils); }
        if self.state.create_use_unity { modules.push(DefaultModule::UnityTest); }

        let build_system = if self.state.create_use_cmake {
            newc_core::project::BuildSystem::CMake
        } else {
            newc_core::project::BuildSystem::Make
        };
        let license = self.state.create_license.as_deref()
            .and_then(newc_core::license::License::from_spdx_id);
        let opts = ScaffoldOptions {
            name: name.clone(),
            author: self.state.create_author.clone(),
            git_init: self.state.create_git,
            modules,
            build_system,
            license,
        };

        match create_project(&opts, &parent) {
            Ok(_) => {
                self.state.create_name.clear();
                self.state.selected_template = None;
                self.open_project(root);
            }
            Err(e) => self.state.set_error(e.to_string()),
        }
    }

    // ── Layout panels ──────────────────────────────────────────────────────────

    fn top_bar(&self) -> Element<'_, Message> {
        // Nav button: highlighted if current view matches
        let is_home     = matches!(self.state.view, View::Home);
        let is_create   = matches!(self.state.view, View::CreateProject);
        let is_lib      = matches!(self.state.view, View::FunctionLibrary);
        let is_cref     = matches!(self.state.view, View::CReference);
        let is_snip     = matches!(self.state.view, View::Snippets);
        let is_settings = matches!(self.state.view, View::Settings);

        let nav = |label: &'static str, active: bool, msg: Message| -> Element<'_, Message> {
            button(text(label).size(13))
                .style(if active { th::btn_nav_active } else { th::btn_nav_inactive })
                .on_press(msg)
                .into()
        };

        let err_count  = self.state.diagnostics.iter().filter(|d| d.is_error()).count();
        let warn_count = self.state.diagnostics.iter().filter(|d| d.kind == diag::DiagKind::Warning).count();
        let base_build_label = match &self.state.build_state {
            BuildState::Idle    => if self.state.build_panel_open { "▼ Build" } else { "▶ Build" },
            BuildState::Running => "⟳ Building",
            BuildState::Done { exit_code: Some(0) } => "✓ Build",
            BuildState::Done { .. } => "✗ Build",
        };
        let build_label = if matches!(self.state.build_state, BuildState::Done { .. }) && (err_count > 0 || warn_count > 0) {
            format!("{base_build_label}  {err_count}E {warn_count}W")
        } else {
            base_build_label.to_string()
        };
        let build_color = match &self.state.build_state {
            BuildState::Running => th::color::YELLOW,
            BuildState::Done { exit_code: Some(0) } => th::color::GREEN,
            BuildState::Done { .. } => th::color::ACCENT,
            _ => th::color::TEXT_DIM,
        };

        let bar = row![
            text("newc").size(15).color(th::color::ACCENT),
            Space::new().width(8),
            nav("Home",     is_home,     Message::Navigate(View::Home)),
            nav("New",      is_create,   Message::Navigate(View::CreateProject)),
            nav("Library",  is_lib,      Message::Navigate(View::FunctionLibrary)),
            nav("CRef",     is_cref,     Message::Navigate(View::CReference)),
            nav("Snippets", is_snip,     Message::Navigate(View::Snippets)),
            nav("Settings", is_settings, Message::Navigate(View::Settings)),
            Space::new().width(Length::Fill),
            button(text(build_label).size(12).color(build_color))
                .style(th::btn_secondary)
                .on_press(Message::ToggleBuildPanel),
        ]
        .align_y(Alignment::Center)
        .spacing(4)
        .padding([6, 10]);

        column![
            container(bar).width(Length::Fill).style(th::deep_style),
            th::separator(),
        ]
        .spacing(0)
        .into()
    }

    fn status_bar(&self) -> Element<'_, Message> {
        use newc_core::git;

        let project_name = self.state.current_project()
            .map(|p| p.name.as_str())
            .unwrap_or("—");

        let git_branch = self.state.current_project()
            .map(|p| git::current_branch(&p.root))
            .unwrap_or_default();
        let branch_str = if git_branch.is_empty() { String::new() } else { format!(" ⎇ {git_branch}") };

        let status_msg = if let Some((s, _)) = &self.state.status { s.as_str() } else { "" };

        column![
            th::separator(),
            container(
                row![
                    text(format!("◆ {project_name}{branch_str}")).size(11).color(th::color::GREEN),
                    Space::new().width(Length::Fill),
                    text(status_msg).size(11).color(th::color::TEXT_DIM),
                    text(format!("v{}", env!("CARGO_PKG_VERSION"))).size(10).color(th::color::TEXT_HINT),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .padding([4, 10])
            )
            .width(Length::Fill)
            .style(th::deep_style),
        ]
        .spacing(0)
        .into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let is_current = |root: &std::path::PathBuf| {
            self.state.current_project().map(|p| &p.root == root).unwrap_or(false)
        };

        let projects: Vec<Element<Message>> = self.state.known_projects
            .iter()
            .filter_map(|p| Some((p.clone(), Project::open(p.clone()).ok()?)))
            .map(|(root, p)| {
                let active = is_current(&root);
                let name = if p.name.len() > 18 {
                    format!("{}…", &p.name[..17])
                } else {
                    p.name.clone()
                };
                let path_short = root.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let _ = path_short; // used below
                container(
                    button(
                        column![
                            text(name).size(12).color(if active { th::color::ACCENT } else { th::color::TEXT }),
                            text(root.to_string_lossy().as_ref().to_string()).size(10).color(th::color::TEXT_HINT),
                        ]
                        .spacing(1)
                    )
                    .style(if active { th::btn_nav_active } else { th::btn_ghost })
                    .on_press(Message::OpenProject(root))
                    .width(Length::Fill)
                )
                .width(Length::Fill)
                .into()
            })
            .collect();

        let list = if projects.is_empty() {
            column![th::hint_text("No projects yet")]
        } else {
            column(projects).spacing(1)
        };

        let sidebar_content = column![
            row![
                th::section_title("Projects"),
                Space::new().width(Length::Fill),
                button(text("+").size(13))
                    .style(th::btn_ghost)
                    .on_press(Message::Navigate(View::CreateProject)),
            ]
            .align_y(Alignment::Center),
            scrollable(list).height(Length::Fill),
            button(text("Browse…").size(12))
                .style(th::btn_secondary)
                .on_press(Message::BrowseForProject)
                .width(Length::Fill),
        ]
        .spacing(8)
        .padding([8, 6]);

        row![
            container(sidebar_content)
                .width(210)
                .height(Length::Fill)
                .style(th::panel_style),
            container(iced::widget::Space::new().width(1).height(Length::Fill))
                .style(|_| iced::widget::container::Style {
                    background: Some(Background::Color(th::color::BORDER_DIM)),
                    ..Default::default()
                }),
        ]
        .into()
    }

    fn central_panel(&self) -> Element<'_, Message> {
        let content: Element<Message> = match &self.state.view {
            View::Home => views::home::view(&self.state),
            View::CreateProject => views::create::view(&self.state),
            View::Settings => views::settings::view(&self.state),

            View::ProjectDetail(p) => views::project::view(&self.state, p),
            View::ProjectStats(p) => views::stats::view(&self.state, p),
            View::ProjectNotes(p) => views::project_notes::view(&self.state, p),
            View::MainBuilder(p) => views::main_builder::view(&self.state, p),
            View::AddModule { project: p, .. } => views::add_module::view(&self.state, p),
            View::GitPanel(p) => views::git_panel::view(&self.state, p),
            View::BuildHistory(p) => views::build_history::view(&self.state, p),
            View::UsageTracker(p) => views::usage_tracker::view(&self.state, p),
            View::MakefileEditor(p) => views::makefile_editor::view(&self.state, p),
            View::ProjectSearch(p) => views::project_search::view(&self.state, p),
            View::HealthDashboard(p) => views::health::view(&self.state, p),

            View::ModuleDetail { project: p, module_name } => {
                if let Some(m) = p.modules.iter().find(|m| &m.name == module_name) {
                    views::module_detail::view(&self.state, module_name, &m.source, &p.root)
                } else {
                    text("Module not found").into()
                }
            }

            View::HeaderEditor { project: p, module_name } => {
                let _ = (p, module_name);
                views::header_editor::view(&self.state)
            }

            View::FunctionLibrary => views::library::view(&self.state, &self.function_lib),
            View::CReference => views::cref::view(&self.state),
            View::Snippets => views::snippets::view(&self.state),
            View::CallGraph(p) => views::call_graph::view(&self.state, p),
            View::DependencyGraph(p) => views::dependency_graph::view(&self.state, p),
            View::FlowChart(p) => views::flow_canvas::view(&self.state, p),
            View::Onboarding(step) => views::onboarding::view(&self.state, *step),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8)
            .into()
    }

}

/// Build the custom Monokai Pro [`Theme`] from hardcoded palette values.
pub fn monokai_pro_theme() -> Theme {
    use iced::theme::{self, Palette};
    use std::sync::Arc;
    Theme::Custom(Arc::new(theme::Custom::new(
        "Monokai Pro".to_string(),
        Palette {
            background: Color::from_rgb8(0x2D, 0x2A, 0x2E),
            text:       Color::from_rgb8(0xFC, 0xFC, 0xFA),
            primary:    Color::from_rgb8(0xFF, 0x61, 0x88),
            success:    Color::from_rgb8(0xA9, 0xDC, 0x76),
            warning:    Color::from_rgb8(0xFF, 0xD8, 0x66),
            danger:     Color::from_rgb8(0xFF, 0x61, 0x88),
        },
    )))
}

/// Resolve a theme name string (as stored in config) to an iced [`Theme`].
///
/// Falls back to `Theme::Dark` for unknown names.
pub fn theme_from_name(name: &str) -> Theme {
    match name {
        "light" => Theme::Light,
        "dracula" => Theme::Dracula,
        "nord" => Theme::Nord,
        "solarized_light" => Theme::SolarizedLight,
        "solarized_dark" => Theme::SolarizedDark,
        "gruvbox_light" => Theme::GruvboxLight,
        "gruvbox_dark" => Theme::GruvboxDark,
        "catppuccin_latte" => Theme::CatppuccinLatte,
        "catppuccin_frappe" => Theme::CatppuccinFrappe,
        "catppuccin_macchiato" => Theme::CatppuccinMacchiato,
        "catppuccin_mocha" => Theme::CatppuccinMocha,
        "tokyo_night" => Theme::TokyoNight,
        "tokyo_night_storm" => Theme::TokyoNightStorm,
        "tokyo_night_light" => Theme::TokyoNightLight,
        "kanagawa_wave" => Theme::KanagawaWave,
        "kanagawa_dragon" => Theme::KanagawaDragon,
        "kanagawa_lotus" => Theme::KanagawaLotus,
        "moonfly" => Theme::Moonfly,
        "nightfly" => Theme::Nightfly,
        "oxocarbon" => Theme::Oxocarbon,
        "ferra" => Theme::Ferra,
        "monokai_pro" => monokai_pro_theme(),
        _ => Theme::Dark,
    }
}

fn file_watch_stream(root: &PathBuf) -> futures::stream::BoxStream<'static, Message> {
    let root = root.clone();
    let (tx, rx) = std::sync::mpsc::channel::<PathBuf>();

    struct WatchState {
        _watcher: Option<notify::RecommendedWatcher>,
        rx: std::sync::mpsc::Receiver<PathBuf>,
    }

    let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(ev) = res {
            for path in ev.paths {
                let _ = tx.send(path);
            }
        }
    });

    let state = WatchState {
        _watcher: watcher.ok().and_then(|mut w| {
            w.watch(&root, notify::RecursiveMode::NonRecursive).ok()?;
            Some(w)
        }),
        rx,
    };

    Box::pin(futures::stream::unfold(state, |state| async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if let Ok(path) = state.rx.try_recv() {
                return Some((Message::FileChanged(path), state));
            }
        }
    }))
}

fn toggle_makefile_flag(content: &str, flag: &str) -> String {
    let ends_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    for line in &mut lines {
        let trimmed = line.trim_start();
        if (trimmed.starts_with("CFLAGS") || trimmed.starts_with("CFLAGS ")) && line.contains('=') {
            if line.contains(flag) {
                *line = line.replace(&format!(" {flag}"), "").replace(&format!("{flag} "), "").replace(flag, "");
            } else {
                line.push(' ');
                line.push_str(flag);
            }
            break;
        }
    }
    let mut out = lines.join("\n");
    if ends_newline { out.push('\n'); }
    out
}

fn scan_for_onboarding_projects() -> Vec<(std::path::PathBuf, bool)> {
    let mut found = Vec::new();
    let Some(home) = dirs::home_dir() else { return found };
    for dir in &["projects", "uni", "Uni", "school", "Documents", "code", "src"] {
        let base = home.join(dir);
        if !base.is_dir() { continue; }
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && newc_core::project::Project::is_newc_project(&p) {
                    found.push((p, true));
                }
            }
        }
    }
    found
}

/// All available themes as `(config_key, display_name)` pairs, in UI order.
pub const ALL_THEMES: &[(&str, &str)] = &[
    ("dark", "Dark"),
    ("light", "Light"),
    ("dracula", "Dracula"),
    ("nord", "Nord"),
    ("solarized_dark", "Solarized Dark"),
    ("solarized_light", "Solarized Light"),
    ("gruvbox_dark", "Gruvbox Dark"),
    ("gruvbox_light", "Gruvbox Light"),
    ("catppuccin_mocha", "Catppuccin Mocha"),
    ("catppuccin_macchiato", "Catppuccin Macchiato"),
    ("catppuccin_frappe", "Catppuccin Frappé"),
    ("catppuccin_latte", "Catppuccin Latte"),
    ("tokyo_night", "Tokyo Night"),
    ("tokyo_night_storm", "Tokyo Night Storm"),
    ("tokyo_night_light", "Tokyo Night Light"),
    ("kanagawa_wave", "Kanagawa Wave"),
    ("kanagawa_dragon", "Kanagawa Dragon"),
    ("kanagawa_lotus", "Kanagawa Lotus"),
    ("moonfly", "Moonfly"),
    ("nightfly", "Nightfly"),
    ("oxocarbon", "Oxocarbon"),
    ("ferra", "Ferra"),
    ("monokai_pro", "Monokai Pro ✦"),
];
