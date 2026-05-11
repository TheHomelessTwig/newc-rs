use newc_core::function_lib::{FunctionLibrary, FunctionTemplate};

#[derive(Default, Clone)]
pub struct LibraryState {
    pub selected: Option<String>,
    pub search: String,
    pub edit_mode: bool,
    pub draft: Option<FunctionTemplate>,
    pub adding_new: bool,
    pub active_group: Option<String>,
    pub rename_input: String,
    pub draft_params: Vec<(String, String)>,
    pub draft_return_type: String,
    pub draft_override_sig: bool,
    pub draft_params_ready: bool,
}

impl LibraryState {
    pub fn new_draft() -> FunctionTemplate {
        FunctionTemplate {
            name: String::new(),
            module: String::new(),
            description: String::new(),
            signature: String::new(),
            header_code: String::new(),
            impl_code: String::new(),
            requires: Vec::new(),
            tags: Vec::new(),
            notes: String::new(),
            starred: false,
        }
    }

    pub fn init_params_from_sig(&mut self, sig: &str) {
        let (ret, params) = parse_sig(sig);
        self.draft_return_type = ret;
        self.draft_params = params;
        self.draft_params_ready = true;
    }

    pub fn reset_builder(&mut self) {
        self.draft_params.clear();
        self.draft_return_type = "void".to_string();
        self.draft_override_sig = false;
        self.draft_params_ready = true;
    }
}

#[derive(Debug, Clone)]
pub enum LibraryAction {
    None,
    Save(FunctionTemplate),
    Delete(String),
    UpdateNotes { name: String, notes: String },
    ToggleStar(String),
    OpenImport,
    CreateGroup { name: String, desc: String },
    RenameGroup { old: String, new: String },
    DeleteGroup { name: String, cascade: bool },
}

pub fn view<'a>(
    _state: &'a crate::state::AppState,
    _lib: &'a FunctionLibrary,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Function Library — porting in progress").into()
}

// ── Signature parser (helper, no egui) ────────────────────────────────────────

fn parse_sig(sig: &str) -> (String, Vec<(String, String)>) {
    let sig = sig.trim();
    let paren = sig.find('(').unwrap_or(sig.len());
    let before_paren = sig[..paren].trim();
    let ret = before_paren
        .rsplit_once(' ')
        .map(|(r, _)| r.trim().to_string())
        .unwrap_or_else(|| "void".to_string());

    let params_str = sig.get(paren + 1..)
        .and_then(|s| s.rfind(')').map(|e| &s[..e]))
        .unwrap_or("").trim();

    let params = if params_str.is_empty() || params_str == "void" {
        Vec::new()
    } else {
        params_str.split(',').map(|p| {
            let p = p.trim();
            if let Some((t, n)) = p.rsplit_once(' ') {
                (t.trim().to_string(), n.trim_start_matches('*').to_string())
            } else {
                (p.to_string(), String::new())
            }
        }).collect()
    };
    (ret, params)
}

