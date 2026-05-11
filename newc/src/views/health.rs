use newc_core::project::Project;

#[derive(Default, Clone)]
pub struct HealthSnapshot {
    pub last_build_ok: bool,
    pub last_build_text: String,
    pub dead_code_count: usize,
    pub dead_code_text: String,
    pub dead_code_funcs: Vec<String>,
    pub missing_includes_count: usize,
    pub missing_includes_text: String,
    pub todos_count: usize,
    pub todos_text: String,
    pub todos: Vec<(String, usize, String)>,
    pub lint_count: usize,
    pub lint_text: String,
    pub lint_warnings: Vec<(String, &'static str, String)>,
    pub header_guard_count: usize,
    pub header_guard_text: String,
    pub header_guard_files: Vec<String>,
    pub proto_mismatch_count: usize,
    pub proto_mismatch_text: String,
    pub proto_mismatches: Vec<(String, String)>,
    pub source_mtime: u64,
}

pub fn view<'a>(
    _state: &'a crate::state::AppState,
    _project: &'a Project,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Health Dashboard — porting in progress").into()
}
