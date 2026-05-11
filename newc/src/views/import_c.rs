use newc_core::function_lib::FunctionTemplate;
use newc_core::sync::ExtractedFunction;

#[derive(Default, Debug)]
pub struct ImportState {
    pub extracted: Vec<ExtractedFunction>,
    pub selected: Vec<bool>,
    pub target_module: String,
    pub path_label: String,
}

impl Clone for ImportState {
    fn clone(&self) -> Self {
        Self {
            extracted: self.extracted.clone(),
            selected: self.selected.clone(),
            target_module: self.target_module.clone(),
            path_label: self.path_label.clone(),
        }
    }
}

pub fn view<'a>(
    _state: &'a crate::state::AppState,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Import C — porting in progress").into()
}

#[allow(dead_code)]
pub fn build_templates(state: &ImportState) -> Vec<FunctionTemplate> {
    state
        .extracted
        .iter()
        .enumerate()
        .filter(|(i, _)| state.selected.get(*i).copied().unwrap_or(false))
        .map(|(_, f)| FunctionTemplate {
            name: f.name.clone(),
            module: state.target_module.clone(),
            description: String::new(),
            signature: f.signature.clone(),
            header_code: f.signature.clone() + ";",
            impl_code: f.body.clone(),
            requires: Vec::new(),
            tags: Vec::new(),
            notes: String::new(),
            starred: false,
        })
        .collect()
}
