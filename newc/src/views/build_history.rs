use newc_core::project::Project;

pub fn view<'a>(
    _state: &'a crate::state::AppState,
    _project: &'a Project,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Build History — porting in progress").into()
}
