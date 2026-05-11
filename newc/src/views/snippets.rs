#[derive(Default, Clone)]
pub struct SnippetsState {
    pub selected_cat: usize,
    pub selected_snippet: Option<usize>,
}

pub fn view<'a>(
    _state: &'a crate::state::AppState,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Snippets — porting in progress").into()
}
