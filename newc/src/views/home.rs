pub fn view<'a>(
    _state: &'a crate::state::AppState,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Home — porting in progress").into()
}
