use std::path::PathBuf;
use std::time::Instant;

use newc_core::{config::AppConfig, function_lib::FunctionLibrary, main_builder::MainBuilderState, project::Project, stats::ProjectStats};
use crate::views::header_editor::HeaderEditorState;
use crate::views::import_c::ImportState;
use crate::views::library::LibraryState;
use crate::views::module_detail::ModuleDetailState;
use crate::views::quick_search::QuickSearchState;
use crate::build_runner::BuildLine;

#[derive(Debug, Clone)]
pub enum View {
    Home,
    CreateProject,
    FunctionLibrary,
    ProjectDetail(Project),
    ProjectStats(Project),
    ModuleDetail { project: Project, module_name: String },
    HeaderEditor { project: Project, module_name: String },
    MainBuilder(Project),
    AddModule { project: Project },
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildState {
    Idle,
    Running,
    Done { exit_code: Option<i32> },
}

pub struct AppState {
    pub view: View,
    pub known_projects: Vec<PathBuf>,
    pub function_lib: FunctionLibrary,
    pub config: AppConfig,
    pub build_lines: Vec<BuildLine>,
    pub build_state: BuildState,
    pub status: Option<(String, Instant)>,
    // Create project form state
    pub create_name: String,
    pub create_git: bool,
    pub create_author: String,
    pub create_include_input: bool,
    pub create_include_math: bool,
    pub create_include_display: bool,
    pub create_include_array: bool,
    // Library view
    pub library_state: LibraryState,
    // Module detail / header editor
    pub module_detail_state: ModuleDetailState,
    pub header_editor_state: HeaderEditorState,
    // Main builder + undo/redo (not persisted — session only)
    pub main_builder: MainBuilderState,
    pub composer_undo: Vec<MainBuilderState>,
    pub composer_redo: Vec<MainBuilderState>,
    // Selected template index in Create view
    pub selected_template: Option<usize>,
    // Import from .c
    pub import_state: ImportState,
    pub show_import: bool,
    // New group form
    pub new_group_name: String,
    pub new_group_desc: String,
    pub show_new_group: bool,
    // Group action modal (rename / delete confirm)
    pub group_action_target: Option<String>,
    pub group_rename_input: String,
    pub delete_group_cascade: bool,
    // Add module form
    pub add_module_name: String,
    // Function picker
    pub func_search: String,
    pub func_selected: Vec<String>,
    // Quick search
    pub quick_search: QuickSearchState,
    // Confirm tidy modal
    pub show_tidy_confirm: bool,
    pub tidy_candidates: Vec<String>,
    // Error modal
    pub error_msg: Option<String>,
    // Cached project stats
    pub cached_stats: Option<(PathBuf, ProjectStats)>,
    // Settings draft (edited copy before save)
    pub config_draft: AppConfig,
}

impl AppState {
    pub fn new() -> Self {
        let author = newc_core::scaffold::detect_author();
        let config = AppConfig::load();
        let config_draft = config.clone();
        Self {
            view: View::Home,
            known_projects: Vec::new(),
            function_lib: FunctionLibrary::load(),
            config,
            build_lines: Vec::new(),
            build_state: BuildState::Idle,
            status: None,
            create_name: String::new(),
            create_git: false,
            create_author: author,
            create_include_input: true,
            create_include_math: true,
            create_include_display: true,
            create_include_array: true,
            library_state: LibraryState::default(),
            module_detail_state: ModuleDetailState::default(),
            header_editor_state: HeaderEditorState::default(),
            main_builder: MainBuilderState::default(),
            composer_undo: Vec::new(),
            composer_redo: Vec::new(),
            selected_template: None,
            import_state: ImportState::default(),
            show_import: false,
            new_group_name: String::new(),
            new_group_desc: String::new(),
            show_new_group: false,
            group_action_target: None,
            group_rename_input: String::new(),
            delete_group_cascade: false,
            add_module_name: String::new(),
            func_search: String::new(),
            func_selected: Vec::new(),
            quick_search: QuickSearchState::default(),
            show_tidy_confirm: false,
            tidy_candidates: Vec::new(),
            error_msg: None,
            cached_stats: None,
            config_draft,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), Instant::now()));
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_msg = Some(msg.into());
    }

    pub fn current_project(&self) -> Option<&Project> {
        match &self.view {
            View::ProjectDetail(p)
            | View::ProjectStats(p)
            | View::MainBuilder(p) => Some(p),
            View::ModuleDetail { project, .. }
            | View::HeaderEditor { project, .. }
            | View::AddModule { project, .. } => Some(project),
            _ => None,
        }
    }

    pub fn get_or_compute_stats(&mut self) -> Option<&ProjectStats> {
        let root = self.current_project()?.root.clone();
        // Recompute if project changed
        if self.cached_stats.as_ref().map(|(p, _)| p) != Some(&root) {
            let s = newc_core::stats::compute(&root);
            self.cached_stats = Some((root, s));
        }
        self.cached_stats.as_ref().map(|(_, s)| s)
    }
}
