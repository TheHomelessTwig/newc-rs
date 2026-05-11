use std::path::PathBuf;

#[derive(Default)]
pub struct QuickSearchState {
    pub open: bool,
    pub query: String,
    pub cursor: usize,
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum QuickSearchResult {
    Function { name: String, module: String, description: String },
    Project { name: String, path: PathBuf },
}

pub fn view<'a>(
    _state: &'a crate::state::AppState,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Quick Search — porting in progress").into()
}
