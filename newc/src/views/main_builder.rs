use egui::{Color32, Key, RichText, ScrollArea, SidePanel, CentralPanel, Ui};
use newc_core::{
    main_builder::{FuncParam, GlobalVar, MainBlock, MainBuilderState, parse_params},
    project::Project,
    sync::extract_function_implementations,
};
use std::collections::HashMap;

pub enum BuilderAction {
    None,
    GoBack,
    WriteMainC,
    /// Snapshot current state for undo before a mutation
    Snapshot,
}

pub fn show(
    ui: &mut Ui,
    project: &Project,
    builder: &mut MainBuilderState,
    author: &str,
    undo_stack: &mut Vec<MainBuilderState>,
    redo_stack: &mut Vec<MainBuilderState>,
) -> BuilderAction {
    let mut action = BuilderAction::None;

    // Ctrl+Z / Ctrl+Y keyboard shortcuts
    ui.input(|i| {
        if i.key_pressed(Key::Z) && i.modifiers.ctrl && !i.modifiers.shift {
            if let Some(prev) = undo_stack.pop() {
                let cur = std::mem::replace(builder, prev);
                redo_stack.push(cur);
            }
        }
        if (i.key_pressed(Key::Y) && i.modifiers.ctrl)
            || (i.key_pressed(Key::Z) && i.modifiers.ctrl && i.modifiers.shift)
        {
            if let Some(next) = redo_stack.pop() {
                let cur = std::mem::replace(builder, next);
                undo_stack.push(cur);
            }
        }
    });

    // Header
    ui.horizontal(|ui| {
        if ui.button("← Project").clicked() {
            action = BuilderAction::GoBack;
        }
        ui.heading(format!("main() Composer — {}", project.name));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(egui::Button::new("Write main.c").fill(Color32::from_rgb(40, 120, 60)))
                .on_hover_text("Overwrite src/main.c with the generated code")
                .clicked()
            {
                action = BuilderAction::WriteMainC;
            }
            // Undo/redo buttons
            if ui.add_enabled(!undo_stack.is_empty(), egui::Button::new("↩ Undo")).clicked() {
                if let Some(prev) = undo_stack.pop() {
                    let cur = std::mem::replace(builder, prev);
                    redo_stack.push(cur);
                }
            }
            if ui.add_enabled(!redo_stack.is_empty(), egui::Button::new("↪ Redo")).clicked() {
                if let Some(next) = redo_stack.pop() {
                    let cur = std::mem::replace(builder, next);
                    undo_stack.push(cur);
                }
            }
            ui.label(RichText::new("Ctrl+Z/Y").small().color(Color32::GRAY));
        });
    });
    ui.separator();

    // Collect func sigs once per frame
    let func_sigs = collect_func_sigs(project);

    // Split: left = controls, right = preview
    SidePanel::left("builder_left")
        .min_width(360.0)
        .max_width(520.0)
        .show_inside(ui, |ui| {
            // Track mutations to push undo snapshot
            let before = builder.clone();
            show_left_panel(ui, project, builder, &func_sigs);
            if *builder != before {
                // State changed — push snapshot
                if undo_stack.last() != Some(&before) {
                    undo_stack.push(before);
                    undo_stack.truncate(50); // cap history
                    redo_stack.clear();
                }
            }
        });

    CentralPanel::default().show_inside(ui, |ui| {
        let date = chrono::Local::now().format("%d/%m/%Y").to_string();
        let preview = builder.preview(author, &date);
        ui.label(RichText::new("Preview — src/main.c").strong());
        ui.separator();
        ScrollArea::vertical().id_salt("builder_preview").show(ui, |ui| {
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                let is_dark = ui.visuals().dark_mode;
                let job = crate::highlight::highlight_c(&preview, is_dark, 12.0);
                ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Extend));
            });
        });
    });

    action
}

fn show_left_panel(
    ui: &mut Ui,
    project: &Project,
    builder: &mut MainBuilderState,
    func_sigs: &HashMap<String, Vec<FuncParam>>,
) {
    ScrollArea::vertical().id_salt("builder_left_scroll").show(ui, |ui| {
        // ── Signature ─────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.checkbox(&mut builder.argc_argv, "int main(int argc, char *argv[])");
        });
        ui.add_space(4.0);

        // ── Includes ──────────────────────────────────────────────────────────
        ui.collapsing("Includes", |ui| {
            let include_dir = project.root.join("include");
            if let Ok(entries) = std::fs::read_dir(&include_dir) {
                let mut names: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("h"))
                    .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().into_owned()))
                    .collect();
                names.sort();
                for name in &names {
                    let mut inc = builder.includes.contains(name);
                    if ui.checkbox(&mut inc, name).changed() {
                        if inc { builder.includes.push(name.clone()); }
                        else { builder.includes.retain(|n| n != name); }
                    }
                }
            }
        });

        // ── Global variables ──────────────────────────────────────────────────
        ui.collapsing("Global Variables", |ui| {
            if ui.small_button("+ Add Global").clicked() {
                builder.globals.push(GlobalVar {
                    type_name: "int".into(), name: "g_var".into(),
                    init: String::new(), is_array: false, array_size: String::new(), is_static: false,
                });
            }
            let mut to_remove: Option<usize> = None;
            for (i, g) in builder.globals.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.small_button("X").clicked() { to_remove = Some(i); }
                    ui.checkbox(&mut g.is_static, "static");
                    ui.add(egui::TextEdit::singleline(&mut g.type_name).desired_width(60.0).hint_text("type"));
                    ui.add(egui::TextEdit::singleline(&mut g.name).desired_width(80.0).hint_text("name"));
                    ui.checkbox(&mut g.is_array, "[]");
                    if g.is_array {
                        ui.add(egui::TextEdit::singleline(&mut g.array_size).desired_width(40.0).hint_text("size"));
                    }
                    ui.label("=");
                    ui.add(egui::TextEdit::singleline(&mut g.init).desired_width(60.0).hint_text("init"));
                });
            }
            if let Some(i) = to_remove { builder.globals.remove(i); }
        });

        ui.separator();

        // ── Block list ────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(RichText::new("main() Blocks").strong());
        });
        ui.horizontal_wrapped(|ui| {
            if ui.small_button("+ Var").clicked() {
                builder.blocks.push(MainBlock::VarDecl {
                    type_name: "int".into(), name: "var".into(),
                    init: String::new(), is_array: false, array_size: String::new(),
                });
            }
            if ui.small_button("+ Call").clicked() {
                builder.blocks.push(MainBlock::FunctionCall {
                    func_name: String::new(), args: Vec::new(),
                    assign_to: String::new(), comment: String::new(),
                });
            }
            if ui.small_button("+ Comment").clicked() { builder.blocks.push(MainBlock::Comment(String::new())); }
            if ui.small_button("+ Raw").clicked() { builder.blocks.push(MainBlock::RawCode(String::new())); }
            if ui.small_button("+ Blank").clicked() { builder.blocks.push(MainBlock::BlankLine); }
            if ui.small_button("+ If").clicked() {
                builder.blocks.push(MainBlock::IfBlock { condition: String::new(), body: Vec::new(), else_body: Vec::new() });
            }
            if ui.small_button("+ While").clicked() {
                builder.blocks.push(MainBlock::WhileLoop { condition: String::new(), body: Vec::new() });
            }
            if ui.small_button("+ For").clicked() {
                builder.blocks.push(MainBlock::ForLoop { init: "int i = 0".into(), condition: "i < n".into(), increment: "i++".into(), body: Vec::new() });
            }
        });
        ui.add_space(4.0);

        let available_funcs: Vec<String> = func_sigs.keys().cloned().collect::<Vec<_>>().tap_sort();

        let mut to_remove: Option<usize> = None;
        let mut to_duplicate: Option<usize> = None;
        let mut move_up: Option<usize> = None;
        let mut move_down: Option<usize> = None;
        let n = builder.blocks.len();

        for i in 0..n {
            let block = &mut builder.blocks[i];
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(40, 40, 50, 200))
                .inner_margin(egui::Margin::same(6))
                .outer_margin(egui::Margin::symmetric(0, 2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(block.label()).small().color(block_label_color(block)));
                        if i > 0 && ui.small_button("Up").clicked() { move_up = Some(i); }
                        if i + 1 < n && ui.small_button("Dn").clicked() { move_down = Some(i); }
                        if ui.small_button("Dup").on_hover_text("Duplicate block").clicked() { to_duplicate = Some(i); }
                        if ui.small_button("X").clicked() { to_remove = Some(i); }
                    });
                    show_block_editor(ui, block, &available_funcs, func_sigs);
                });
        }

        if let Some(i) = to_remove { if i < builder.blocks.len() { builder.blocks.remove(i); } }
        if let Some(i) = to_duplicate { if i < builder.blocks.len() { let b = builder.blocks[i].clone(); builder.blocks.insert(i + 1, b); } }
        if let Some(i) = move_up { if i > 0 && i < builder.blocks.len() { builder.blocks.swap(i - 1, i); } }
        if let Some(i) = move_down { if i + 1 < builder.blocks.len() { builder.blocks.swap(i, i + 1); } }
    });
}

fn show_block_editor(
    ui: &mut Ui,
    block: &mut MainBlock,
    funcs: &[String],
    func_sigs: &HashMap<String, Vec<FuncParam>>,
) {
    match block {
        MainBlock::VarDecl { type_name, name, init, is_array, array_size } => {
            egui::Grid::new(ui.next_auto_id()).num_columns(2).show(ui, |ui| {
                ui.label("Type:"); ui.text_edit_singleline(type_name); ui.end_row();
                ui.label("Name:"); ui.text_edit_singleline(name); ui.end_row();
                ui.label("Array:"); ui.checkbox(is_array, ""); ui.end_row();
                if *is_array { ui.label("Size:"); ui.text_edit_singleline(array_size); ui.end_row(); }
                ui.label("Init:"); ui.text_edit_singleline(init); ui.end_row();
            });
            ui.label(RichText::new(format!("→ {}", block_preview(block))).small().monospace().color(Color32::LIGHT_GRAY));
        }

        MainBlock::FunctionCall { func_name, args, assign_to, comment } => {
            // Function picker
            ui.horizontal(|ui| {
                ui.label("Function:");
                egui::ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(if func_name.is_empty() { "— pick —" } else { func_name.as_str() })
                    .show_ui(ui, |ui| {
                        for f in funcs {
                            if ui.selectable_value(func_name, f.clone(), f).clicked() {
                                // Resize args to match signature param count
                                if let Some(params) = func_sigs.get(f) {
                                    let needed = params.len();
                                    args.resize(needed, String::new());
                                }
                            }
                        }
                    });
                ui.text_edit_singleline(func_name); // free-text fallback
            });

            // Param-by-param argument fields
            if let Some(params) = func_sigs.get(func_name.as_str()) {
                if !params.is_empty() {
                    ui.label(RichText::new("Arguments:").small().strong());
                    args.resize(params.len(), String::new());
                    egui::Grid::new(ui.next_auto_id()).num_columns(2).show(ui, |ui| {
                        for (j, param) in params.iter().enumerate() {
                            let label = format!("{} {}:", param.type_name, param.name);
                            ui.label(RichText::new(&label).small().monospace().color(Color32::GRAY));
                            ui.text_edit_singleline(&mut args[j]);
                            ui.end_row();
                        }
                    });
                }
            } else {
                // Unknown function — show args as add/remove list
                ui.label(RichText::new("Arguments:").small().strong());
                let mut to_remove: Option<usize> = None;
                for (j, arg) in args.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.small_button("−").clicked() { to_remove = Some(j); }
                        ui.add(egui::TextEdit::singleline(arg).desired_width(f32::INFINITY).hint_text(format!("arg {}", j + 1)));
                    });
                }
                if let Some(j) = to_remove { args.remove(j); }
                if ui.small_button("+ arg").clicked() { args.push(String::new()); }
            }

            ui.horizontal(|ui| {
                ui.label("→ assign to:"); ui.text_edit_singleline(assign_to);
            });
            ui.horizontal(|ui| {
                ui.label("Comment:"); ui.text_edit_singleline(comment);
            });
            ui.label(RichText::new(format!("→ {}", block_preview(block))).small().monospace().color(Color32::LIGHT_GRAY));
        }

        MainBlock::Comment(text) => {
            ui.add(egui::TextEdit::singleline(text).desired_width(f32::INFINITY).hint_text("Comment text…"));
        }
        MainBlock::RawCode(text) => {
            ui.add(egui::TextEdit::multiline(text).code_editor().desired_rows(3).desired_width(f32::INFINITY).hint_text("Raw C code…"));
        }
        MainBlock::BlankLine => {
            ui.label(RichText::new("(blank line)").color(Color32::GRAY).small());
        }
        MainBlock::IfBlock { condition, body, else_body } => {
            ui.horizontal(|ui| {
                ui.label("if (");
                ui.add(egui::TextEdit::singleline(condition).hint_text("x > 0").desired_width(160.0));
                ui.label(") {");
            });
            show_nested_blocks(ui, body, funcs, func_sigs, "if_body");
            if ui.small_button("+ Add to body").clicked() {
                body.push(MainBlock::BlankLine);
            }
            ui.collapsing("} else {", |ui| {
                show_nested_blocks(ui, else_body, funcs, func_sigs, "else_body");
                if ui.small_button("+ Add to else").clicked() {
                    else_body.push(MainBlock::BlankLine);
                }
            });
        }
        MainBlock::WhileLoop { condition, body } => {
            ui.horizontal(|ui| {
                ui.label("while (");
                ui.add(egui::TextEdit::singleline(condition).hint_text("i < n").desired_width(160.0));
                ui.label(") {");
            });
            show_nested_blocks(ui, body, funcs, func_sigs, "while_body");
            if ui.small_button("+ Add to body").clicked() {
                body.push(MainBlock::BlankLine);
            }
        }
        MainBlock::ForLoop { init, condition, increment, body } => {
            egui::Grid::new(ui.next_auto_id()).num_columns(2).show(ui, |ui| {
                ui.label("Init:"); ui.add(egui::TextEdit::singleline(init).hint_text("int i = 0")); ui.end_row();
                ui.label("Condition:"); ui.add(egui::TextEdit::singleline(condition).hint_text("i < n")); ui.end_row();
                ui.label("Increment:"); ui.add(egui::TextEdit::singleline(increment).hint_text("i++")); ui.end_row();
            });
            show_nested_blocks(ui, body, funcs, func_sigs, "for_body");
            if ui.small_button("+ Add to body").clicked() {
                body.push(MainBlock::BlankLine);
            }
        }
    }
}

fn show_nested_blocks(
    ui: &mut Ui,
    blocks: &mut Vec<MainBlock>,
    funcs: &[String],
    func_sigs: &HashMap<String, Vec<FuncParam>>,
    id_salt: &str,
) {
    let mut to_remove: Option<usize> = None;
    let mut move_up: Option<usize> = None;
    let mut move_down: Option<usize> = None;
    let n = blocks.len();

    if n == 0 {
        ui.label(RichText::new("  (empty)").small().color(Color32::GRAY));
    }
    for i in 0..n {
        egui::Frame::new()
            .fill(Color32::from_rgba_premultiplied(30, 40, 60, 200))
            .inner_margin(egui::Margin::same(4))
            .outer_margin(egui::Margin { left: 16, top: 1, right: 0, bottom: 1 })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(blocks[i].label()).small().color(block_label_color(&blocks[i])));
                    if i > 0 && ui.small_button("Up").clicked() { move_up = Some(i); }
                    if i + 1 < n && ui.small_button("Dn").clicked() { move_down = Some(i); }
                    if ui.small_button("X").clicked() { to_remove = Some(i); }
                });
                show_block_editor(ui, &mut blocks[i], funcs, func_sigs);
            });
    }
    if let Some(i) = to_remove { blocks.remove(i); }
    if let Some(i) = move_up { blocks.swap(i - 1, i); }
    if let Some(i) = move_down { blocks.swap(i, i + 1); }
    let _ = id_salt; // used for future DnD id
}

fn block_label_color(block: &MainBlock) -> Color32 {
    match block {
        MainBlock::VarDecl { .. }      => Color32::from_rgb(100, 180, 255),
        MainBlock::FunctionCall { .. } => Color32::from_rgb(100, 220, 130),
        MainBlock::Comment(_)          => Color32::from_rgb(180, 180, 100),
        MainBlock::RawCode(_)          => Color32::from_rgb(200, 130, 100),
        MainBlock::BlankLine           => Color32::GRAY,
        MainBlock::IfBlock { .. }      => Color32::from_rgb(200, 120, 220),
        MainBlock::WhileLoop { .. }    => Color32::from_rgb(220, 160, 80),
        MainBlock::ForLoop { .. }      => Color32::from_rgb(220, 130, 80),
    }
}

fn block_preview(block: &MainBlock) -> String {
    block.to_c().trim_start_matches('\t').to_string()
}

fn collect_func_sigs(project: &Project) -> HashMap<String, Vec<FuncParam>> {
    let src_dir = project.root.join("src");
    let Ok(entries) = std::fs::read_dir(&src_dir) else { return HashMap::new() };
    let mut map = HashMap::new();
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("c") && e.file_name() != "main.c")
        .map(|e| e.path())
        .collect();
    paths.sort();
    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for f in extract_function_implementations(&content) {
                let params = parse_params(&f.signature);
                map.insert(f.name, params);
            }
        }
    }
    map
}

// Helper trait to sort a Vec in-place and return it
trait TapSort { fn tap_sort(self) -> Self; }
impl TapSort for Vec<String> {
    fn tap_sort(mut self) -> Self { self.sort(); self }
}
