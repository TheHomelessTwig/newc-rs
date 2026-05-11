use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct ModuleDetailState {
    pub selected_func: Option<String>,
    pub edit_mode: bool,
    pub edit_buf: String,
    pub show_header_editor: bool,
    pub unreachable_funcs: Vec<String>,
    pub check_ran: bool,
    pub proto_mismatch: Option<String>,
    pub show_call_tree: bool,
    pub call_tree_lines: Vec<String>,
    pub highlight_line: Option<usize>,
}

pub fn view<'a>(
    _state: &'a crate::state::AppState,
    _module_name: &str,
    _src_path: &PathBuf,
    _project_root: &Path,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Module Detail — porting in progress").into()
}
