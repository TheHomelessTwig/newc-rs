//! Application-wide state types: [`AppState`], [`View`], [`Message`],
//! [`BuildState`], [`Toast`], and persistence helpers for known/recent projects.
//!
//! [`AppState`] is the single mutable store owned by [`crate::app::NewcApp`].
//! [`View`] determines which panel is rendered in the central area.
//! [`Message`] is the exhaustive set of events that drive `update`.

use std::path::PathBuf;
use std::time::Instant;
use iced::widget::{pane_grid, text_editor};
use iced::window;

use newc_core::{
    config::{AppConfig, ProjectConfig}, diag::Diagnostic,
    grep::SearchResult, main_builder::MainBuilderState,
    meta::ProjectMeta, project::Project, stats::ProjectStats,
    function_lib::FunctionTemplate,
};
use crate::views::header_editor::HeaderEditorState;
use crate::views::import_c::ImportState;
use crate::views::library::{LibraryPane, LibraryState};
use crate::views::cref::CRefPane;
use crate::views::snippets::SnippetsPane;
use crate::views::module_detail::{ModuleDetailState, ModulePane};
use crate::views::quick_search::QuickSearchState;
use crate::build_runner::BuildLine;

/// Which view is currently displayed in the central panel.
///
/// Variants that carry a [`Project`] embed a cloned snapshot of the project at
/// the time of navigation; call [`AppState::current_project`] to get a
/// reference from the active view.
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

/// Lifecycle state of the background `make` process.
#[derive(Debug, Clone, PartialEq)]
pub enum BuildState {
    /// No build has been started or the runner is between builds.
    Idle,
    /// A `make` subprocess is running; output is being streamed.
    Running,
    /// The subprocess exited. `exit_code` is `None` if the process was killed
    /// or could not be waited on.
    Done { exit_code: Option<i32> },
}

/// All events that can be sent to [`crate::app::NewcApp::update`].
///
/// Variants are grouped by feature area with inline comments. For large feature
/// areas (composer, module detail, library) the groups are delimited by comment
/// banners. Individual message variants are self-describing by name and payload;
/// see [`crate::app::NewcApp::update`] for the handling of each one.
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
    BuildArgsChanged(String),
    BuildProfileSelect(Option<String>),
    DiagTabRaw(bool),
    DiagNavPrev,
    DiagNavNext,
    SplitCompareSelect(Option<String>),
    ModuleHoverRequest(String),

    // Create project form
    CreateName(String),
    CreateAuthor(String),
    CreateLocation(String),
    CreateLocationBrowse,
    CreateGitToggle(bool),
    CreateUseCmakeToggle(bool),
    CreateInclude(String, bool),
    CreateTemplate(usize),
    /// SPDX id to license the new project under, or `None` for no license.
    CreateLicense(Option<String>),
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
    /// Navigate directly to a CRef function from the quick-search overlay.
    QuickSearchSelectFunc { name: String, header: String },
    QuickSearchClose,

    // Settings
    SettingsSave,
    SettingsDiscard,
    SettingsDraftEditor(String),
    SettingsDraftTerminal(String),
    SettingsDraftTheme(String),
    SettingsDraftClangStyle(String),
    SettingsDraftCodeFontSize(f32),
    // Theme — applied immediately (no save needed)
    ThemeSelect(String),

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

    // Pane resize events
    LibraryPaneResized(pane_grid::ResizeEvent),
    CRefPaneResized(pane_grid::ResizeEvent),
    SnippetsPaneResized(pane_grid::ResizeEvent),
    ModulePaneResized(pane_grid::ResizeEvent),

    // Git
    GitCommitMsg(String),
    GitStage(String),
    GitUnstage(String),
    GitCommit,
    GitPull,
    GitPush,
    GitStash,
    GitStashPop,
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
    MakefileToggleFlag(String),

    // Refactoring: rename function
    ModuleRenameStart(String),
    ModuleRenameInput(String),
    ModuleRenameSubmit,

    // Refactoring: move function to module
    ModuleMoveStart(String),
    ModuleMoveTargetInput(String),
    ModuleMoveSubmit,

    // Doxygen stub generation
    ModuleGenerateDoc(String),

    // Lint quick-fix
    LintQuickFix { module: String, function: String, line_no: usize, code: String },

    // GDB launcher
    LaunchDebugger,

    // Per-project settings
    ProjectConfigDraftEditor(String),
    ProjectConfigDraftTerminal(String),
    ProjectConfigDraftClangStyle(String),
    ProfileNameInput(String),
    ProfileCflagsInput(String),
    ProfileAdd,
    ProfileRemove(String),
    ProjectConfigSave,
    ProjectConfigClear,

    // Project actions (wired from project.rs dead buttons)
    OpenInEditor,
    ExportZip,
    ExportCompileCommands,
    SubmissionStudentInput(String),
    SubmissionAssignmentInput(String),
    PackSubmission,
    GenerateReport,
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
    SearchReplaceInput(String),
    SearchReplacePreview,
    SearchReplaceApply,

    // Usage tracker
    UsageSearch(String),

    // Main builder block manipulation
    ComposerAddBlock(newc_core::main_builder::MainBlock),
    ComposerBlockMoveUp(usize),
    ComposerBlockMoveDown(usize),
    ComposerBlockDelete(usize),
    ComposerBlockDuplicate(usize),
    ComposerSelectBlock(usize),
    ComposerEditField { block_index: usize, field: String, value: String },
    ComposerAddChildBlock { parent: usize, block: newc_core::main_builder::MainBlock },
    ComposerDeleteChildBlock { parent: usize, child: usize },
    ComposerMoveChildUp { parent: usize, child: usize },
    ComposerMoveChildDown { parent: usize, child: usize },
    ComposerToggleInclude(String),
    ComposerDragStart(usize),
    ComposerDragDrop(usize),
    ComposerDragEnd,
    ComposerUndo,
    ComposerRedo,
    ComposerWriteMainC,

    // Module detail
    ModuleSelectFunc(Option<String>),
    ModuleEditMode(bool),
    ModuleEditAction(text_editor::Action),
    ModuleSaveFunc { name: String, new_impl: String },
    ModuleDeleteFunc(String),
    ModuleRunCheck,
    ModuleShowCallTree(bool),
    ModuleAddFromLibrary,
    ModuleSyncNow,
    ModuleClangFormat,

    // Header editor
    HeaderContent(String),
    HeaderEditorAction(text_editor::Action),
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
    UpdateCheckResult(Option<String>),
    UpdateInstallDone,

    // Git
    GitInit,

    // Module delete confirm
    ModuleDeleteFuncConfirm,

    // Library → module insert
    LibraryInsertToModule,

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

/// A transient notification displayed in the bottom-right corner of the window.
///
/// Toasts auto-dismiss after [`Toast::duration_ms`] milliseconds; the timer is
/// advanced by [`Message::ToastTick`] at 100 ms intervals.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    /// Milliseconds elapsed since the toast was created.
    pub elapsed_ms: u32,
    /// Total display duration before auto-dismiss.
    pub duration_ms: u32,
}

/// Visual severity of a [`Toast`] — controls border colour and background tint.
#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

impl Toast {
    /// Create a success toast (auto-dismisses after 3 s).
    pub fn success(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ToastKind::Success, elapsed_ms: 0, duration_ms: 3000 }
    }
    /// Create an error toast (auto-dismisses after 5 s).
    pub fn error(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ToastKind::Error, elapsed_ms: 0, duration_ms: 5000 }
    }
    /// Create an info toast (auto-dismisses after 2.5 s).
    pub fn info(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), kind: ToastKind::Info, elapsed_ms: 0, duration_ms: 2500 }
    }
    /// Returns `true` when the toast's elapsed time has reached its duration.
    pub fn is_expired(&self) -> bool {
        self.elapsed_ms >= self.duration_ms
    }
}

/// Identifies which field of a library function draft is being edited.
///
/// Used by [`Message::LibraryDraftField`] to route text-input changes to the
/// correct field without needing a separate message variant per field.
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

/// Which part of a function parameter is being edited in the library draft form.
#[derive(Debug, Clone)]
pub enum ParamPart {
    Type,
    Name,
}

/// All mutable UI and domain state for the running application.
///
/// Owned exclusively by [`crate::app::NewcApp`] and passed by reference to
/// every view function. Fields are grouped by feature area with inline comments
/// in the source. Only non-obvious fields are documented here; the rest are
/// self-describing by name.
pub struct AppState {
    /// The currently active view (determines what the central panel renders).
    pub view: View,
    /// All projects the user has opened or imported, persisted across sessions.
    pub known_projects: Vec<PathBuf>,
    pub config: AppConfig,
    /// Theme name string matching a key in [`crate::app::ALL_THEMES`].
    pub active_theme: String,
    /// Per-project config override loaded from `.newc_config.toml`, if present.
    pub project_config: Option<ProjectConfig>,
    /// Draft copy of the per-project config being edited before save.
    pub project_config_draft: ProjectConfig,
    pub profile_name_input: String,
    pub profile_cflags_input: String,
    /// Accumulated output lines from the current or most recent build.
    pub build_lines: Vec<BuildLine>,
    /// Parsed `vg.xml` errors after a `valgrind-xml` build target run.
    pub valgrind_errors: Vec<newc_core::valgrind::ValgrindError>,
    pub build_state: BuildState,
    /// Status bar message and the [`Instant`] it was set (auto-cleared after 4 s).
    pub status: Option<(String, Instant)>,
    // Create project form state
    pub create_name: String,
    pub create_git: bool,
    /// `true` to scaffold a CMakeLists.txt instead of a Makefile.
    pub create_use_cmake: bool,
    pub create_author: String,
    pub create_include_input: bool,
    pub create_include_math: bool,
    pub create_include_display: bool,
    pub create_include_array: bool,
    pub create_include_strings: bool,
    pub create_include_linked_list: bool,
    pub create_include_files: bool,
    pub create_include_test_utils: bool,
    /// `true` to scaffold the Unity-style test harness instead of `test_utils`.
    pub create_use_unity: bool,
    /// SPDX id of the license to apply, or `None` for no license file.
    pub create_license: Option<String>,
    pub create_location: String,
    // Module detail / header editor
    pub module_detail_state: ModuleDetailState,
    pub header_editor_state: HeaderEditorState,
    // Main builder + undo/redo (not persisted — session only)
    pub main_builder: MainBuilderState,
    /// Undo stack for the main.c composer (capped at 50 entries).
    pub composer_undo: Vec<MainBuilderState>,
    /// Redo stack for the main.c composer (capped at 50 entries).
    pub composer_redo: Vec<MainBuilderState>,
    /// Index of the currently selected composer block, if any.
    pub composer_selected: Option<usize>,
    /// Index of the block being dragged in the composer, if any.
    pub composer_drag: Option<usize>,
    // Selected template index in Create view
    pub selected_template: Option<usize>,
    // Import from .c
    pub import_state: ImportState,
    pub show_import: bool,
    // Pane grid state (adjustable column splits)
    pub library_panes: pane_grid::State<LibraryPane>,
    pub cref_panes: pane_grid::State<CRefPane>,
    pub snippets_panes: pane_grid::State<SnippetsPane>,
    pub module_panes: pane_grid::State<ModulePane>,
    // Library state (formerly in SharedTools)
    pub library_state: LibraryState,
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
    // Refactor modals
    pub rename_func_old: String,
    pub rename_func_input: String,
    pub show_rename_modal: bool,
    pub move_func_name: String,
    pub move_func_target_input: String,
    pub show_move_modal: bool,
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
    // Assignment submission packer
    pub submission_student_input: String,
    pub submission_assignment_input: String,
    /// Module name to insert into when the user navigates from ModuleDetail to Library.
    pub pending_library_insert_module: Option<String>,
    /// Non-empty when the error modal should be displayed.
    pub error_msg: Option<String>,
    /// Cached `(project_root, stats)` — invalidated whenever the project changes.
    pub cached_stats: Option<(PathBuf, ProjectStats)>,
    /// In-progress edits of the global config, committed on `SettingsSave`.
    pub config_draft: AppConfig,
    // Project notes (content loaded per-project)
    pub notes_content: text_editor::Content,
    pub notes_dirty: bool,
    // Shortcuts modal
    pub show_shortcuts: bool,
    /// `true` on first launch (no `.onboarded` marker in the config dir).
    pub is_first_run: bool,
    /// Projects discovered during onboarding scan as `(path, selected)` pairs.
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
    /// CLI args passed to the program when running the `run` target.
    pub build_run_args: String,
    /// Name of the currently selected named build profile (see `ProjectConfig::build_profiles`).
    pub build_profile_active: Option<String>,
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
    pub search_replace: String,
    pub replace_preview: Vec<newc_core::grep::ReplacePreview>,
    // Compiler diagnostics
    pub diagnostics: Vec<Diagnostic>,
    /// Index into `diagnostics` for the Prev/Next build-error navigation buttons.
    pub diag_nav_index: usize,
    /// Other module name to show side-by-side in module_detail's split view, if any.
    pub split_compare_module: Option<String>,
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
    /// Construct `AppState` with defaults and values loaded from disk.
    ///
    /// Reads the global config, the known-projects list, and recent projects.
    /// Detects first-run by checking for `~/.config/newc/.onboarded`.
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
            project_config: None,
            project_config_draft: ProjectConfig::default(),
            profile_name_input: String::new(),
            profile_cflags_input: String::new(),
            build_lines: Vec::new(),
            valgrind_errors: Vec::new(),
            build_state: BuildState::Idle,
            status: None,
            create_name: String::new(),
            create_git: false,
            create_use_cmake: false,
            create_author: author,
            create_include_input: true,
            create_include_math: true,
            create_include_display: true,
            create_include_array: true,
            create_include_strings: false,
            create_include_linked_list: false,
            create_include_files: false,
            create_include_test_utils: false,
            create_use_unity: false,
            create_license: None,
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
            library_panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.15,
                a: Box::new(pane_grid::Configuration::Pane(LibraryPane::Groups)),
                b: Box::new(pane_grid::Configuration::Split {
                    axis: pane_grid::Axis::Vertical,
                    ratio: 0.24,
                    a: Box::new(pane_grid::Configuration::Pane(LibraryPane::Functions)),
                    b: Box::new(pane_grid::Configuration::Pane(LibraryPane::Detail)),
                }),
            }),
            cref_panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.16,
                a: Box::new(pane_grid::Configuration::Pane(CRefPane::Headers)),
                b: Box::new(pane_grid::Configuration::Split {
                    axis: pane_grid::Axis::Vertical,
                    ratio: 0.22,
                    a: Box::new(pane_grid::Configuration::Pane(CRefPane::Functions)),
                    b: Box::new(pane_grid::Configuration::Pane(CRefPane::Detail)),
                }),
            }),
            snippets_panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.28,
                a: Box::new(pane_grid::Configuration::Pane(SnippetsPane::List)),
                b: Box::new(pane_grid::Configuration::Pane(SnippetsPane::Code)),
            }),
            module_panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.20,
                a: Box::new(pane_grid::Configuration::Pane(ModulePane::Sidebar)),
                b: Box::new(pane_grid::Configuration::Pane(ModulePane::Panel)),
            }),
            library_state: LibraryState::default(),
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
            rename_func_old: String::new(),
            rename_func_input: String::new(),
            show_rename_modal: false,
            move_func_name: String::new(),
            move_func_target_input: String::new(),
            show_move_modal: false,
            add_module_name: String::new(),
            func_search: String::new(),
            func_selected: Vec::new(),
            quick_search: QuickSearchState::default(),
            show_tidy_confirm: false,
            tidy_candidates: Vec::new(),
            submission_student_input: String::new(),
            submission_assignment_input: String::new(),
            pending_library_insert_module: None,
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
            build_run_args: String::new(),
            build_profile_active: None,
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
            search_replace: String::new(),
            replace_preview: Vec::new(),
            diagnostics: Vec::new(),
            diag_nav_index: 0,
            split_compare_module: None,
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

    /// Append a [`Toast`] to the notification queue.
    pub fn push_toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    /// Set a status bar message (auto-clears after 4 s via `StatusTick`).
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), Instant::now()));
    }

    /// Display an error modal with the given message.
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_msg = Some(msg.into());
    }

    /// Return the [`Project`] embedded in the current view, if any.
    ///
    /// Returns `None` for global views (Home, Settings, FunctionLibrary, etc.)
    /// that are not scoped to a specific project.
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

    /// Return cached project stats, computing them if the cache is stale or empty.
    pub fn get_or_compute_stats(&mut self) -> Option<&ProjectStats> {
        let root = self.current_project()?.root.clone();
        if self.cached_stats.as_ref().map(|(cached_root, _)| cached_root) != Some(&root) {
            let computed_stats = newc_core::stats::compute(&root);
            self.cached_stats = Some((root, computed_stats));
        }
        self.cached_stats.as_ref().map(|(_, stats)| stats)
    }
}

// ── Known-projects persistence ────────────────────────────────────────────────

fn projects_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("projects.toml"))
}

/// Load the persisted known-project list from `~/.config/newc/projects.toml`.
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

/// Load the persisted recent-projects list from `~/.config/newc/recents.toml`.
pub fn load_recent_projects() -> Vec<PathBuf> {
    let Some(path) = recents_path() else { return Vec::new() };
    let Ok(content) = std::fs::read_to_string(&path) else { return Vec::new() };
    #[derive(serde::Deserialize)]
    struct RecentsList { recent: Vec<PathBuf> }
    toml::from_str::<RecentsList>(&content)
        .map(|r| r.recent)
        .unwrap_or_default()
}

/// Persist the recent-projects list to `~/.config/newc/recents.toml`.
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

/// Persist the known-projects list to `~/.config/newc/projects.toml`.
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
