use newc_core::function_lib::FunctionLibrary;

pub fn view<'a>(
    _state: &'a crate::state::AppState,
    _lib: &'a FunctionLibrary,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Function Picker — porting in progress").into()
}
