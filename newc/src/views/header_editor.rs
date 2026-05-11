#[derive(Default, Clone)]
pub struct StructField {
    pub type_name: String,
    pub field_name: String,
    pub comment: String,
}

#[derive(Default, Clone)]
pub struct StructBuilder {
    pub struct_name: String,
    pub fields: Vec<StructField>,
    pub typedef: bool,
}

impl StructBuilder {
    pub fn to_code(&self) -> String {
        let mut out = String::new();
        if self.typedef {
            out.push_str(&format!("typedef struct {} {{\n", self.struct_name));
        } else {
            out.push_str(&format!("struct {} {{\n", self.struct_name));
        }
        for f in &self.fields {
            if f.comment.is_empty() {
                out.push_str(&format!("    {} {};\n", f.type_name, f.field_name));
            } else {
                out.push_str(&format!(
                    "    {} {}; /* {} */\n",
                    f.type_name, f.field_name, f.comment
                ));
            }
        }
        if self.typedef {
            out.push_str(&format!("}} {};\n", self.struct_name));
        } else {
            out.push_str("};\n");
        }
        out
    }
}

#[derive(Default, Clone)]
pub struct HeaderEditorState {
    pub content: String,
    pub insert_name: String,
    pub insert_type: String,
    pub insert_value: String,
    pub dirty: bool,
    pub struct_builder: StructBuilder,
}

pub fn view<'a>(
    _state: &'a crate::state::AppState,
) -> iced::Element<'a, crate::state::Message> {
    iced::widget::text("Header Editor — porting in progress").into()
}
