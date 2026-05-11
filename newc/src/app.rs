use std::path::PathBuf;
use std::time::Duration;

use iced::{Element, Subscription, Task, Theme};
use iced::widget::{column, row, text, button, container, scrollable, Space};
use iced::{Alignment, Length, Color};
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

pub struct NewcApp {
    state: AppState,
    runner: BuildRunner,
    function_lib: FunctionLibrary,
}

impl NewcApp {
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

        let runner = BuildRunner::spawn();
        let function_lib = FunctionLibrary::load();

        let mut app = Self { state, runner, function_lib };

        if let Some(path) = initial_path {
            app.open_project(path);
        }

        (app, Task::none())
    }

    pub fn title(&self) -> String {
        match &self.state.view {
            View::ProjectDetail(p) | View::ProjectStats(p) | View::ModuleDetail { project: p, .. } => {
                format!("newc — {}", p.name)
            }
            _ => String::from("newc"),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Drain any buffered build output first
        self.drain_build_output();

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
                        // Load existing main.c into composer
                        self.state.main_builder =
                            newc_core::main_builder::MainBuilderState::load_from_main_c(&project.root);
                        if self.state.create_author.is_empty() {
                            self.state.create_author = newc_core::scaffold::detect_author();
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
                    self.state.build_target_current = target.clone();
                    self.state.build_state = BuildState::Running;
                    self.state.build_lines.clear();
                    self.state.diagnostics.clear();
                    self.runner.run(&target, cwd);
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
            }

            Message::BuildAutoScrollToggle => {
                self.state.build_auto_scroll = !self.state.build_auto_scroll;
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
                _ => {}
            },

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

            Message::QuickSearchSelect(_) => {
                // Handled per-view during porting
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

            // Apply theme immediately without requiring settings save
            Message::ThemeSelect(name) => {
                self.state.active_theme = name.clone();
                self.state.config.theme = name.clone();
                self.state.config_draft.theme = name;
                let _ = self.state.config.save();
            }

            Message::LibraryToggleOpen => {
                self.state.show_library = !self.state.show_library;
            }

            Message::CRefToggleOpen => {
                self.state.show_cref = !self.state.show_cref;
            }

            Message::SnippetsToggleOpen => {
                self.state.show_snippets = !self.state.show_snippets;
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
                    let _ = notes::save(&project.root, &self.state.notes_content);
                    self.state.notes_dirty = false;
                    self.state.set_status("Notes saved.");
                }
            }

            Message::NotesContent(s) => {
                self.state.notes_content = s;
                self.state.notes_dirty = true;
            }

            Message::MakefileSave => {
                if let Some(project) = self.state.current_project() {
                    let path = project.root.join("Makefile");
                    if let Err(e) = std::fs::write(&path, &self.state.makefile_content) {
                        self.state.set_error(e.to_string());
                    } else {
                        self.state.makefile_dirty = false;
                        self.state.set_status("Makefile saved.");
                    }
                }
            }

            Message::MakefileContent(s) => {
                self.state.makefile_content = s;
                self.state.makefile_dirty = true;
            }

            Message::GitCommitMsg(s) => self.state.git_commit_msg = s,
            Message::GitNewBranch(s) => self.state.git_new_branch = s,
            Message::GitShowDiff(v) => self.state.git_show_diff = v,
            Message::GitDiffStaged(v) => self.state.git_diff_staged = v,

            Message::GitCommit => {
                if let Some(project) = self.state.current_project() {
                    let root = project.root.clone();
                    let msg = self.state.git_commit_msg.trim().to_string();
                    if msg.is_empty() { return Task::none(); }
                    match newc_core::git::commit(&root, &msg) {
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

            Message::SearchQuery(s) => self.state.search_query = s,
            Message::SearchSubmit => {
                if let Some(project) = self.state.current_project() {
                    let query = self.state.search_query.trim().to_string();
                    if !query.is_empty() {
                        self.state.search_results = newc_core::grep::search(&project.root, &query);
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
                // TODO: wire up save_template logic during views::project port
            }

            Message::ShowTidyConfirm(v) => self.state.show_tidy_confirm = v,
            Message::TidyConfirm => {
                // TODO: wire up tidy logic
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

            Message::UpdateCheck | Message::UpdateInstall(_) => {
                // Updater will be wired in during standalone features phase
            }

            Message::DiagJumpTo { module, line } => {
                if let Some(project) = self.state.current_project().cloned() {
                    self.state.module_detail_state.highlight_line = Some(line);
                    self.state.view = View::ModuleDetail {
                        project,
                        module_name: module,
                    };
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
                // Clipboard access via iced::clipboard::write
                return iced::clipboard::write(code);
            }

            // Main builder
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
                    if let Some(fname) = &self.state.module_detail_state.selected_func.clone() {
                        if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                            if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                                if let Ok(src) = std::fs::read_to_string(&m.source) {
                                    let funcs = newc_core::sync::extract_function_implementations(&src);
                                    if let Some(f) = funcs.iter().find(|f| &f.name == fname) {
                                        self.state.module_detail_state.edit_buf = f.body.clone();
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
            Message::ModuleDeleteFunc(_name) => {
                // Handled in future iteration — requires confirmation modal
            }
            Message::ModuleRunCheck => {
                if let View::ModuleDetail { project, module_name: _ } = &self.state.view.clone() {
                    let funcs = newc_core::analysis::check(&project.root).unwrap_or_default();
                    self.state.module_detail_state.unreachable_funcs = funcs.into_iter().map(|f| f.name).collect();
                    self.state.module_detail_state.check_ran = true;
                }
            }
            Message::ModuleShowCallTree(v) => self.state.module_detail_state.show_call_tree = v,
            Message::ModuleAddFromLibrary => { /* TODO */ }
            Message::ModuleSyncNow => {
                if let View::ModuleDetail { project, module_name } = &self.state.view.clone() {
                    let _ = newc_core::sync::sync_module(&project.root, module_name);
                    self.state.set_status(format!("Synced {module_name}.h"));
                }
            }
            Message::ModuleClangFormat => { /* TODO */ }

            // Header
            Message::HeaderSave => {
                if let View::HeaderEditor { project, module_name } = &self.state.view.clone() {
                    if let Some(m) = project.modules.iter().find(|m| &m.name == module_name) {
                        let _ = newc_core::header::write_ignore_block(&m.header, &self.state.header_editor_state.content);
                        self.state.header_editor_state.dirty = false;
                        self.state.set_status("Header saved.");
                    }
                }
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

            Message::None => {}
            _ => {}
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
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

        // Overlay: shortcuts
        if self.state.show_shortcuts {
            central = container(
                column![
                    views::shortcuts::view(&self.state),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        // Overlay: quick search
        if self.state.quick_search.open {
            central = container(
                views::quick_search::view(&self.state),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        let content = row![sidebar, central]
            .width(Length::Fill)
            .height(Length::Fill);

        let mut layout = column![top_bar, content];

        if self.state.build_panel_open {
            layout = layout.push(views::build_panel::view(&self.state));
        }

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme(&self) -> Theme {
        theme_from_name(&self.state.active_theme)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Status message auto-clear: tick every second, clear if > 4 seconds old
        if self.state.status.is_some() {
            iced::time::every(Duration::from_secs(1))
                .map(|_| Message::None)
        } else {
            Subscription::none()
        }
    }
}

// ── Private helpers ────────────────────────────────────────────────────────────

impl NewcApp {
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
                self.state.notes_content = notes::load(&p.root);
                self.state.notes_dirty = false;
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

        let opts = ScaffoldOptions {
            name: name.clone(),
            author: self.state.create_author.clone(),
            git_init: self.state.create_git,
            modules,
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
        let nav_btn = |label: &'static str, msg: Message| {
            button(text(label)).on_press(msg)
        };

        let status_text = if let Some((s, _)) = &self.state.status {
            text(s.as_str())
        } else {
            text("")
        };

        let bar = row![
            nav_btn("Home", Message::Navigate(View::Home)),
            nav_btn("New", Message::Navigate(View::CreateProject)),
            nav_btn("Library", Message::Navigate(View::FunctionLibrary)),
            nav_btn("CRef", Message::Navigate(View::CReference)),
            nav_btn("Snippets", Message::Navigate(View::Snippets)),
            nav_btn("Settings", Message::Navigate(View::Settings)),
            Space::new().width(Length::Fill),
            status_text,
            button(text(if self.state.build_panel_open { "Hide Build" } else { "Build" }))
                .on_press(Message::ToggleBuildPanel),
        ]
        .align_y(Alignment::Center)
        .spacing(6)
        .padding(8);

        container(bar)
            .width(Length::Fill)
            .into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let projects: Vec<Element<Message>> = self.state.known_projects
            .iter()
            .filter_map(|p| Project::open(p.clone()).ok())
            .map(|p| {
                let name = p.name.clone();
                let root = p.root.clone();
                button(text(name))
                    .on_press(Message::OpenProject(root))
                    .width(Length::Fill)
                    .into()
            })
            .collect();

        let list = if projects.is_empty() {
            column![text("No projects").size(12)]
        } else {
            column(projects).spacing(2)
        };

        let sidebar_content = column![
            text("Projects").size(13),
            scrollable(list).height(Length::Fill),
            button(text("Browse…")).on_press(Message::BrowseForProject),
        ]
        .spacing(8)
        .padding(8);

        container(sidebar_content)
            .width(200)
            .height(Length::Fill)
            .into()
    }

    fn central_panel(&self) -> Element<'_, Message> {
        let content: Element<Message> = match &self.state.view {
            View::Home => views::home::view(&self.state),
            View::CreateProject => views::create::view(&self.state),
            View::Settings => views::settings::view(&self.state),

            View::ProjectDetail(p) => views::project::view(&self.state, p),
            View::ProjectStats(p) => views::stats::view(&self.state, p),
            View::ProjectNotes(_p) => text("Notes — porting in progress").into(),
            View::MainBuilder(p) => views::main_builder::view(&self.state, p),
            View::AddModule { project: p, .. } => views::add_module::view(&self.state, p),
            View::GitPanel(p) => views::git_panel::view(&self.state, p),
            View::BuildHistory(p) => views::build_history::view(&self.state, p),
            View::UsageTracker(p) => views::usage_tracker::view(&self.state, p),
            View::MakefileEditor(_p) => text("Makefile Editor — porting in progress").into(),
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
        };

        // Drawer overlays (library, cref, snippets)
        // TODO: implement as side drawers in Phase 1 Step 6

        container(
            scrollable(content).height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .into()
    }

    fn build_output_panel(&self) -> Element<'_, Message> {
        use crate::build_runner::LineKind;

        let lines: Vec<Element<Message>> = self.state.build_lines.iter().map(|line| {
            let color = match line.kind {
                LineKind::Stderr => Color::from_rgb(1.0, 0.4, 0.4),
                LineKind::Info => Color::from_rgb(0.6, 0.8, 1.0),
                LineKind::Done { exit_code: Some(0), .. } => Color::from_rgb(0.4, 1.0, 0.4),
                LineKind::Done { .. } => Color::from_rgb(1.0, 0.4, 0.4),
                _ => Color::WHITE,
            };
            text(line.text.as_str()).color(color).size(12).into()
        }).collect();

        let log = if lines.is_empty() {
            column![text("No build output.").size(12)]
        } else {
            column(lines).spacing(2)
        };

        let controls = row![
            button(text("Clear")).on_press(Message::BuildPanelClear),
            button(text(if self.state.build_auto_scroll { "Auto-scroll ✓" } else { "Auto-scroll" }))
                .on_press(Message::BuildAutoScrollToggle),
            button(text("Kill")).on_press(Message::BuildKill),
            Space::new().width(Length::Fill),
            button(text("×")).on_press(Message::ToggleBuildPanel),
        ]
        .spacing(4)
        .padding([4, 8]);

        let panel = column![
            controls,
            scrollable(log).height(Length::Fixed(200.0)),
        ];

        container(panel)
            .width(Length::Fill)
            .into()
    }

    fn error_modal<'a>(&'a self, msg: &'a str) -> Element<'a, Message> {
        column![
            text("Error").size(16),
            text(msg),
            button(text("Dismiss")).on_press(Message::ErrorDismiss),
        ]
        .spacing(8)
        .padding(16)
        .into()
    }
}

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
        _ => Theme::Dark, // default "dark" + any unknown
    }
}

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
    ("nightfly", "Nightfly (≈Monokai)"),
    ("oxocarbon", "Oxocarbon"),
    ("ferra", "Ferra"),
];
