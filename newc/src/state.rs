use std::path::PathBuf;
use std::time::Instant;
use iced::widget::text_editor;
use iced::window;

use newc_core::{
    config::AppConfig, diag::Diagnostic,
    grep::SearchResult, main_builder::MainBuilderState,
    meta::ProjectMeta, project::Project, stats::ProjectStats,
    function_lib::FunctionTemplate,
};
use crate::views::header_editor::HeaderEditorState;
use crate::views::import_c::ImportState;
use crate::views::module_detail::ModuleDetailState;
use crate::views::quick_search::QuickSearchState;
use crate::views::library::LibraryState;
use crate::build_runner::BuildLine;

#[derive(Debug, Clone)]
pub enum View {
    Home,
    CreateProject,
    FunctionLibrary,
    CReference,
    Snippets,
    ProjectDetail(Project),
    ProjectStats(Project),
    ProjectNotes(Project),
    ModuleDetail { project: Project, module_name: String },
    HeaderEditor { project: Project, module_name: String },
    MainBuilder(Project),
    AddModule { project: Project },
    GitPanel(Project),
    BuildHistory(Project),
    UsageTracker(Project),
    MakefileEditor(Project),
    ProjectSearch(Project),
    HealthDashboard(Project),
    CallGraph(Project),
    DependencyGraph(Project),
    FlowChart(Project),
    Settings,
    Onboarding(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildState {
    Idle,
    Running,
    Done { exit_code: Option<i32> },
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    Navigate(View),

    // Project management
    OpenProject(PathBuf),
    AddKnownProject(PathBuf),
    BrowseForProject,
    RefreshProject,

    // Build
    BuildStart(String),
    BuildKill,
    BuildLine(BuildLine),
    ToggleBuildPanel,
    BuildPanelClear,
    BuildAutoScrollToggle,
    DiagTabRaw(bool),

    // Create project form
    CreateName(String),
    CreateAuthor(String),
    CreateLocation(String),
    CreateLocationBrowse,
    CreateGitToggle(bool),
    CreateInclude(String, bool),
    CreateTemplate(usize),
    CreateSubmit,

    // Module actions
    AddModuleName(String),
    AddModuleSubmit,
    RemoveModule(String),
    ConfirmRemoveModule,
    CancelRemoveModule,

    // Quick search
    QuickSearchToggle,
    QuickSearchQuery(String),
    QuickSearchCursor(usize),
    QuickSearchSelect(usize),
    QuickSearchClose,

    // Settings
    SettingsSave,
    SettingsDiscard,
    SettingsDraftEditor(String),
    SettingsDraftTerminal(String),
    SettingsDraftTheme(String),
    SettingsDraftClangStyle(String),
    // Theme — applied immediately (no save needed)
    ThemeSelect(String),

    // Library
    LibraryAction(crate::views::library::LibraryAction),
    LibraryToggleOpen,
    CRefToggleOpen,
    SnippetsToggleOpen,

    // Library state mutations
    LibrarySearch(String),
    LibrarySelect(Option<String>),
    LibraryGroupSelect(Option<String>),
    LibraryEditMode(bool),
    LibraryAddingNew(bool),
    LibraryDraftField(LibraryField, String),
    LibraryDraftParam(usize, ParamPart, String),
    LibraryAddParam,
    LibraryRemoveParam(usize),
    LibraryDraftReturnType(String),
    LibraryDraftOverrideSig(bool),
    LibrarySave(FunctionTemplate),
    LibraryDelete(String),
    LibraryToggleStar(String),
    LibraryUpdateNotes { name: String, notes: String },
    LibraryGroupNew,
    LibraryGroupDelete(String),

    // CRef
    CRefSearch(String),
    CRefSelectHeader(Option<String>),
    CRefSelectFunc(Option<&'static str>),

    // Snippets
    SnippetsCat(usize),
    SnippetsSelect(Option<usize>),
    SnippetsCopy(String),

    // Git
    GitCommitMsg(String),
    GitStage(String),
    GitUnstage(String),
    GitCommit,
    GitPull,
    GitPush,
    GitNewBranch(String),
    GitCreateBranch,
    GitCheckout(String),
    GitDeleteBranch(String),
    GitShowDiff(bool),
    GitDiffStaged(bool),

    // Notes
    NotesEdit(text_editor::Action),
    NotesSave,

    // Makefile editor
    MakefileEdit(text_editor::Action),
    MakefileSave,

    // Project actions (wired from project.rs dead buttons)
    OpenInEditor,
    ExportZip,
    RunCheck,
    SyncAll,
    SyncModule(String),

    // File watcher
    FileChanged(PathBuf),

    // Onboarding wizard
    OnboardingNext,
    OnboardingBack,
    OnboardingToggleProject(usize),
    OnboardingFinish,

    // Multi-window management
    OpenLibraryWindow,
    OpenCRefWindow,
    OpenSnippetsWindow,
    WindowClosed(window::Id),

    // Search
    SearchQuery(String),
    SearchSubmit,

    // Usage tracker
    UsageSearch(String),

    // Main builder block manipulation
    ComposerAddBlock(newc_core::main_builder::MainBlock),
    ComposerBlockMoveUp(usize),
    ComposerBlockMoveDown(usize),
    ComposerBlockDelete(usize),
    ComposerSelectBlock(usize),
    ComposerEditField { idx: usize, field: String, value: String },
    ComposerDragStart(usize),
    ComposerDragDrop(usize),
    ComposerDragEnd,
    ComposerUndo,
    ComposerRedo,
    ComposerWriteMainC,

    // Module detail
    ModuleSelectFunc(Option<String>),
    ModuleEditMode(bool),
    ModuleEditBuf(String),
    ModuleSaveFunc { name: String, new_impl: String },
    ModuleDeleteFunc(String),
    ModuleRunCheck,
    ModuleShowCallTree(bool),
    ModuleAddFromLibrary,
    ModuleSyncNow,
    ModuleClangFormat,

    // Header editor
    HeaderContent(String),
    HeaderSave,

    // Import
    ImportPickFile,
    ImportExtracted(crate::views::import_c::ImportState),
    ImportToggleFunc(usize),
    ImportTargetModule(String),
    ImportSubmit,

    // Meta
    MetaShowEditor(bool),
    MetaCourse(String),
    MetaVersion(String),
    MetaSave,

    // Save as template
    ShowSaveTemplate(bool),
    SaveTemplateName(String),
    SaveTemplateDesc(String),
    SaveTemplateSubmit,

    // Tidy confirm
    ShowTidyConfirm(bool),
    TidyConfirm,

    // Workspaces
    WorkspaceSelect(Option<String>),
    WorkspaceInput(String),
    WorkspaceNew,
    WorkspaceCancelNew,
    ShowArchivedToggle,
    MoveToWorkspace(PathBuf),

    // UI modals
    ErrorDismiss,
    ShowShortcuts(bool),
    ShowImport(bool),
    ShowNewGroup(bool),
    NewGroupName(String),
    NewGroupDesc(String),
    NewGroupSubmit,
    GroupActionTarget(Option<String>),
    GroupRenameInput(String),
    GroupRenameSubmit,
    GroupDeleteCascade(bool),
    GroupDeleteSubmit,

    // Update
    UpdateCheck,
    UpdateInstall(String),

    // Build-panel diagnostics click
    DiagJumpTo { module: String, line: usize },

    // Subscription ticks
    PollBuildOutput,
    StatusTick,
    ToastTick,
    ToastDismiss(usize),

    // Graph canvas interaction
    GraphNodeSelect(String),
    GraphPan { dx: f32, dy: f32 },
    GraphZoom(f32),
    GraphReset,
    GraphExport,

    None,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub elapsed_ms: u32,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

impl Toast {
    pub fn success(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ToastKind::Success, elapsed_ms: 0, duration_ms: 3000 }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ToastKind::Error, elapsed_ms: 0, duration_ms: 5000 }
    }
    pub fn info(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ToastKind::Info, elapsed_ms: 0, duration_ms: 2500 }
    }
    pub fn is_expired(&self) -> bool {
        self.elapsed_ms >= self.duration_ms
    }
}

#[derive(Debug, Clone)]
pub enum LibraryField {
    Name,
    Module,
    Description,
    Signature,
    Header,
    Impl,
    Tags,
    Notes,
}

#[derive(Debug, Clone)]
pub enum ParamPart {
    Type,
    Name,
}

pub struct AppState {
    pub view: View,
    pub known_projects: Vec<PathBuf>,
    pub config: AppConfig,
    pub active_theme: String,
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
    pub create_include_strings: bool,
    pub create_include_linked_list: bool,
    pub create_include_files: bool,
    pub create_include_test_utils: bool,
    pub create_location: String,
    // Module detail / header editor
    pub module_detail_state: ModuleDetailState,
    pub header_editor_state: HeaderEditorState,
    // Main builder + undo/redo (not persisted — session only)
    pub main_builder: MainBuilderState,
    pub composer_undo: Vec<MainBuilderState>,
    pub composer_redo: Vec<MainBuilderState>,
    pub composer_selected: Option<usize>,
    pub composer_drag: Option<usize>,
    // Selected template index in Create view
    pub selected_template: Option<usize>,
    // Import from .c
    pub import_state: ImportState,
    pub show_import: bool,
    // Library state (formerly in SharedTools)
    pub library_state: LibraryState,
    pub show_library: bool,
    pub show_cref: bool,
    pub show_snippets: bool,
    pub cref_search: String,
    pub cref_selected_header: Option<String>,
    pub cref_selected_func: Option<&'static str>,
    pub snippets_cat: usize,
    pub snippets_selected: Option<usize>,
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
    // Function picker (used by add_module and create views)
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
    // Project notes (content loaded per-project)
    pub notes_content: text_editor::Content,
    pub notes_dirty: bool,
    // Shortcuts modal
    pub show_shortcuts: bool,
    // First-run onboarding
    pub is_first_run: bool,
    pub onboarding_found: Vec<(PathBuf, bool)>,
    // Window IDs for multi-window
    pub main_window: Option<window::Id>,
    pub library_window: Option<window::Id>,
    pub cref_window: Option<window::Id>,
    pub snippets_window: Option<window::Id>,
    // Confirm module removal
    pub confirm_remove_module: Option<(Project, String)>,
    // Git panel
    pub git_commit_msg: String,
    pub git_new_branch: String,
    pub git_show_diff: bool,
    pub git_diff_staged: bool,
    // Save-as-template modal
    pub show_save_template_modal: bool,
    pub save_template_name: String,
    pub save_template_desc: String,
    // Project metadata
    pub show_meta_editor: bool,
    pub meta_draft: ProjectMeta,
    // Build target currently running (for history)
    pub build_target_current: String,
    pub build_panel_open: bool,
    pub build_auto_scroll: bool,
    pub build_panel_open_hint: bool,
    // Workspaces
    pub active_workspace: Option<String>,
    pub show_archived: bool,
    pub workspace_input: String,
    pub show_new_workspace: bool,
    pub move_to_workspace_project: Option<PathBuf>,
    // Makefile editor
    pub makefile_content: text_editor::Content,
    pub makefile_dirty: bool,
    // Usage tracker search
    pub usage_search: String,
    // Project search
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    // Compiler diagnostics
    pub diagnostics: Vec<Diagnostic>,
    pub diag_tab_raw: bool,
    // Health dashboard
    pub health_computed: bool,
    pub health_snapshot: crate::views::health::HealthSnapshot,
    // Recent projects (last 5)
    pub recent_projects: Vec<PathBuf>,
    // Toast notifications
    pub toasts: Vec<Toast>,
    // Graph canvas state
    pub graph_selected: Option<String>,
    pub graph_pan_x: f32,
    pub graph_pan_y: f32,
    pub graph_zoom: f32,
}

impl AppState {
    pub fn new() -> Self {
        let author = newc_core::scaffold::detect_author();
        let config = AppConfig::load();
        let config_draft = config.clone();
        let is_first_run = dirs::config_dir()
            .map(|d| !d.join("newc").join(".onboarded").exists())
            .unwrap_or(false);
        let known_projects = load_known_projects();
        let active_theme = config.theme.clone();
        Self {
            view: View::Home,
            known_projects,
            active_theme,
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
            create_include_strings: false,
            create_include_linked_list: false,
            create_include_files: false,
            create_include_test_utils: false,
            create_location: dirs::home_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            module_detail_state: ModuleDetailState::default(),
            header_editor_state: HeaderEditorState::default(),
            main_builder: MainBuilderState::default(),
            composer_undo: Vec::new(),
            composer_redo: Vec::new(),
            composer_selected: None,
            composer_drag: None,
            selected_template: None,
            import_state: ImportState::default(),
            show_import: false,
            library_state: LibraryState::default(),
            show_library: false,
            show_cref: false,
            show_snippets: false,
            cref_search: String::new(),
            cref_selected_header: None,
            cref_selected_func: None,
            snippets_cat: 0,
            snippets_selected: None,
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
            notes_content: text_editor::Content::new(),
            notes_dirty: false,
            show_shortcuts: false,
            is_first_run,
            onboarding_found: Vec::new(),
            main_window: None,
            library_window: None,
            cref_window: None,
            snippets_window: None,
            confirm_remove_module: None,
            git_commit_msg: String::new(),
            git_new_branch: String::new(),
            git_show_diff: false,
            git_diff_staged: false,
            show_save_template_modal: false,
            save_template_name: String::new(),
            save_template_desc: String::new(),
            show_meta_editor: false,
            meta_draft: ProjectMeta::default(),
            build_target_current: String::new(),
            build_panel_open: true,
            build_auto_scroll: true,
            build_panel_open_hint: false,
            active_workspace: None,
            show_archived: false,
            workspace_input: String::new(),
            show_new_workspace: false,
            move_to_workspace_project: None,
            makefile_content: text_editor::Content::new(),
            makefile_dirty: false,
            usage_search: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            diagnostics: Vec::new(),
            diag_tab_raw: true,
            health_computed: false,
            health_snapshot: crate::views::health::HealthSnapshot::default(),
            recent_projects: load_recent_projects(),
            toasts: Vec::new(),
            graph_selected: None,
            graph_pan_x: 0.0,
            graph_pan_y: 0.0,
            graph_zoom: 1.0,
        }
    }

    pub fn push_toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
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
            | View::ProjectNotes(p)
            | View::MainBuilder(p)
            | View::GitPanel(p)
            | View::BuildHistory(p)
            | View::UsageTracker(p)
            | View::MakefileEditor(p)
            | View::ProjectSearch(p)
            | View::HealthDashboard(p)
            | View::CallGraph(p)
            | View::DependencyGraph(p)
            | View::FlowChart(p) => Some(p),
            View::ModuleDetail { project, .. }
            | View::HeaderEditor { project, .. }
            | View::AddModule { project, .. } => Some(project),
            _ => None,
        }
    }

    pub fn get_or_compute_stats(&mut self) -> Option<&ProjectStats> {
        let root = self.current_project()?.root.clone();
        if self.cached_stats.as_ref().map(|(p, _)| p) != Some(&root) {
            let s = newc_core::stats::compute(&root);
            self.cached_stats = Some((root, s));
        }
        self.cached_stats.as_ref().map(|(_, s)| s)
    }
}

// ── Known-projects persistence ────────────────────────────────────────────────

fn projects_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("projects.toml"))
}

pub fn load_known_projects() -> Vec<PathBuf> {
    let Some(path) = projects_path() else { return Vec::new() };
    let Ok(content) = std::fs::read_to_string(&path) else { return Vec::new() };
    #[derive(serde::Deserialize)]
    struct ProjectsList { projects: Vec<PathBuf> }
    toml::from_str::<ProjectsList>(&content)
        .map(|pl| pl.projects)
        .unwrap_or_default()
}

fn recents_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("recents.toml"))
}

pub fn load_recent_projects() -> Vec<PathBuf> {
    let Some(path) = recents_path() else { return Vec::new() };
    let Ok(content) = std::fs::read_to_string(&path) else { return Vec::new() };
    #[derive(serde::Deserialize)]
    struct RecentsList { recent: Vec<PathBuf> }
    toml::from_str::<RecentsList>(&content)
        .map(|r| r.recent)
        .unwrap_or_default()
}

pub fn save_recent_projects(recent: &[PathBuf]) {
    let Some(path) = recents_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    #[derive(serde::Serialize)]
    struct RecentsList<'a> { recent: &'a [PathBuf] }
    if let Ok(content) = toml::to_string_pretty(&RecentsList { recent }) {
        let _ = std::fs::write(path, content);
    }
}

pub fn save_known_projects(projects: &[PathBuf]) {
    let Some(path) = projects_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    #[derive(serde::Serialize)]
    struct ProjectsList<'a> { projects: &'a [PathBuf] }
    if let Ok(content) = toml::to_string_pretty(&ProjectsList { projects }) {
        let _ = std::fs::write(path, content);
    }
}
