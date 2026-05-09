use egui::{Color32, RichText, ScrollArea, Ui};
use newc_core::header;
use std::path::PathBuf;

pub struct HeaderEditorState {
    pub content: String,
    pub loaded: bool,
    pub insert_name: String,
    pub insert_value: String,
    pub insert_type: String,
}

impl Default for HeaderEditorState {
    fn default() -> Self {
        Self {
            content: String::new(),
            loaded: false,
            insert_name: String::new(),
            insert_value: String::new(),
            insert_type: String::new(),
        }
    }
}

pub enum HeaderEditorAction {
    None,
    Save,
    Close,
}

pub fn show(
    ui: &mut Ui,
    hdr_path: &PathBuf,
    module_name: &str,
    state: &mut HeaderEditorState,
) -> HeaderEditorAction {
    let mut action = HeaderEditorAction::None;

    // Load on first show
    if !state.loaded {
        state.content = header::read_ignore_block(hdr_path).unwrap_or_default();
        state.loaded = true;
    }

    ui.horizontal(|ui| {
        ui.heading(format!("Header: {module_name}.h"));
        ui.label(
            RichText::new("Editing SYNC_IGNORE block — structs, enums, defines, etc.")
                .small()
                .color(Color32::GRAY),
        );
    });
    ui.separator();

    // Insert helper toolbar
    ui.horizontal(|ui| {
        ui.label(RichText::new("Insert:").strong());

        if ui.button("Struct").clicked() {
            let name = if state.insert_name.is_empty() { "MyStruct" } else { &state.insert_name };
            state.content.push_str(&format!("\n{}", header::struct_template(name)));
        }
        if ui.button("Enum").clicked() {
            let name = if state.insert_name.is_empty() { "MyEnum" } else { &state.insert_name };
            state.content.push_str(&format!("\n{}", header::enum_template(name)));
        }
        if ui.button("#define").clicked() {
            let name = if state.insert_name.is_empty() { "MY_DEFINE" } else { &state.insert_name };
            let val = if state.insert_value.is_empty() { "0" } else { &state.insert_value };
            state.content.push_str(&format!("\n{}", header::define_template(name, val)));
        }
        if ui.button("Typedef").clicked() {
            let orig = if state.insert_type.is_empty() { "int" } else { &state.insert_type };
            let alias = if state.insert_name.is_empty() { "MyType" } else { &state.insert_name };
            state.content.push_str(&format!("\n{}", header::typedef_template(orig, alias)));
        }
        if ui.button("Constant").clicked() {
            let ty = if state.insert_type.is_empty() { "int" } else { &state.insert_type };
            let name = if state.insert_name.is_empty() { "MY_CONST" } else { &state.insert_name };
            let val = if state.insert_value.is_empty() { "0" } else { &state.insert_value };
            state.content.push_str(&format!("\n{}", header::constant_template(ty, name, val)));
        }
    });

    // Quick-fill fields for the helper buttons
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.add(egui::TextEdit::singleline(&mut state.insert_name).desired_width(120.0));
        ui.label("Type:");
        ui.add(egui::TextEdit::singleline(&mut state.insert_type).desired_width(80.0));
        ui.label("Value:");
        ui.add(egui::TextEdit::singleline(&mut state.insert_value).desired_width(80.0));
    });
    ui.separator();

    // Main editor
    ui.label(
        RichText::new("Content inside SYNC_IGNORE block (preserved across 'sync'):").strong(),
    );
    ScrollArea::vertical().id_salt("hdr_edit_scroll").max_height(400.0).show(ui, |ui| {
        ui.add(
            egui::TextEdit::multiline(&mut state.content)
                .code_editor()
                .desired_rows(20)
                .desired_width(f32::INFINITY),
        );
    });

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            action = HeaderEditorAction::Save;
        }
        if ui.button("Close without saving").clicked() {
            action = HeaderEditorAction::Close;
        }
    });
    ui.add_space(4.0);
    ui.label(
        RichText::new("Prototypes are managed automatically by Sync — don't add them here.")
            .small()
            .color(Color32::GRAY),
    );

    action
}
