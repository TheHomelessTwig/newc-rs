use egui::{Color32, RichText, ScrollArea, SidePanel, CentralPanel, Ui};
use newc_core::{
    main_builder::{MainBlock, MainBuilderState},
    project::Project,
    sync::extract_function_implementations,
};

pub enum BuilderAction {
    None,
    GoBack,
    WriteMainC,
}

pub fn show(
    ui: &mut Ui,
    project: &Project,
    builder: &mut MainBuilderState,
    author: &str,
) -> BuilderAction {
    let mut action = BuilderAction::None;

    // Header
    ui.horizontal(|ui| {
        if ui.button("← Project").clicked() {
            action = BuilderAction::GoBack;
        }
        ui.heading(format!("Main Builder — {}", project.name));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(egui::Button::new("Write to main.c").fill(Color32::from_rgb(40, 120, 60)))
                .on_hover_text("Overwrite src/main.c with the generated code")
                .clicked()
            {
                action = BuilderAction::WriteMainC;
            }
        });
    });
    ui.separator();

    // Split: left = block list + controls, right = preview
    SidePanel::left("builder_left")
        .min_width(320.0)
        .max_width(480.0)
        .show_inside(ui, |ui| {
            show_block_list(ui, project, builder);
        });

    CentralPanel::default().show_inside(ui, |ui| {
        let date = chrono::Local::now().format("%d/%m/%Y").to_string();
        let preview = builder.preview(author, &date);
        ui.label(RichText::new("Preview — src/main.c").strong());
        ui.separator();
        ScrollArea::vertical().id_salt("builder_preview").show(ui, |ui| {
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                ui.add(
                    egui::Label::new(RichText::new(&preview).monospace().size(12.0))
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
            });
        });
    });

    action
}

fn show_block_list(ui: &mut Ui, project: &Project, builder: &mut MainBuilderState) {
    // Include toggles
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
                let mut included = builder.includes.contains(name);
                if ui.checkbox(&mut included, name).changed() {
                    if included {
                        builder.includes.push(name.clone());
                    } else {
                        builder.includes.retain(|n| n != name);
                    }
                }
            }
        }
    });

    ui.separator();
    ui.label(RichText::new("Blocks (drag up/down to reorder)").strong());

    // Add block toolbar
    ui.horizontal(|ui| {
        if ui.small_button("+ Var").clicked() {
            builder.blocks.push(MainBlock::VarDecl {
                type_name: "int".to_string(),
                name: "var".to_string(),
                init: String::new(),
                is_array: false,
                array_size: String::new(),
            });
        }
        if ui.small_button("+ Call").clicked() {
            builder.blocks.push(MainBlock::FunctionCall {
                func_name: String::new(),
                args: Vec::new(),
                assign_to: String::new(),
                comment: String::new(),
            });
        }
        if ui.small_button("+ Comment").clicked() {
            builder.blocks.push(MainBlock::Comment(String::new()));
        }
        if ui.small_button("+ Raw").clicked() {
            builder.blocks.push(MainBlock::RawCode(String::new()));
        }
        if ui.small_button("+ Blank").clicked() {
            builder.blocks.push(MainBlock::BlankLine);
        }
    });
    ui.add_space(4.0);

    // Collect available function names from project modules
    let available_funcs = collect_project_funcs(project);

    let mut to_remove: Option<usize> = None;
    let mut move_up: Option<usize> = None;
    let mut move_down: Option<usize> = None;

    ScrollArea::vertical().id_salt("builder_blocks").show(ui, |ui| {
        let n = builder.blocks.len();
        for i in 0..n {
            let block = &mut builder.blocks[i];
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(40, 40, 50, 200))
                .inner_margin(egui::Margin::same(6))
                .outer_margin(egui::Margin::symmetric(0, 2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(block.label())
                                .small()
                                .color(block_label_color(block)),
                        );
                        if i > 0 && ui.small_button("↑").clicked() {
                            move_up = Some(i);
                        }
                        if i + 1 < n && ui.small_button("↓").clicked() {
                            move_down = Some(i);
                        }
                        if ui.small_button("✕").clicked() {
                            to_remove = Some(i);
                        }
                    });

                    show_block_editor(ui, block, &available_funcs);
                });
        }
    });

    if let Some(i) = to_remove {
        builder.blocks.remove(i);
    }
    if let Some(i) = move_up {
        builder.blocks.swap(i - 1, i);
    }
    if let Some(i) = move_down {
        builder.blocks.swap(i, i + 1);
    }
}

fn show_block_editor(ui: &mut Ui, block: &mut MainBlock, funcs: &[String]) {
    match block {
        MainBlock::VarDecl { type_name, name, init, is_array, array_size } => {
            egui::Grid::new(ui.next_auto_id()).num_columns(2).show(ui, |ui| {
                ui.label("Type:");
                ui.text_edit_singleline(type_name);
                ui.end_row();
                ui.label("Name:");
                ui.text_edit_singleline(name);
                ui.end_row();
                ui.label("Array:");
                ui.checkbox(is_array, "");
                ui.end_row();
                if *is_array {
                    ui.label("Size:");
                    ui.text_edit_singleline(array_size);
                    ui.end_row();
                }
                ui.label("Init:");
                ui.text_edit_singleline(init);
                ui.end_row();
            });
            ui.label(
                RichText::new(format!("→ {}", block_preview(block)))
                    .small()
                    .color(Color32::LIGHT_GRAY)
                    .monospace(),
            );
        }

        MainBlock::FunctionCall { func_name, args, assign_to, comment } => {
            ui.horizontal(|ui| {
                ui.label("Function:");
                egui::ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(if func_name.is_empty() { "— pick —" } else { func_name.as_str() })
                    .show_ui(ui, |ui| {
                        for f in funcs {
                            ui.selectable_value(func_name, f.clone(), f);
                        }
                    });
                // Free-text fallback
                ui.text_edit_singleline(func_name);
            });
            ui.label("Arguments (one per line):");
            let mut args_text = args.join("\n");
            if ui
                .add(
                    egui::TextEdit::multiline(&mut args_text)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY)
                        .hint_text("arg1\narg2\n..."),
                )
                .changed()
            {
                *args = args_text
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
            }
            ui.horizontal(|ui| {
                ui.label("Assign to:");
                ui.text_edit_singleline(assign_to);
            });
            ui.horizontal(|ui| {
                ui.label("Comment:");
                ui.text_edit_singleline(comment);
            });
            ui.label(
                RichText::new(format!("→ {}", block_preview(block)))
                    .small()
                    .color(Color32::LIGHT_GRAY)
                    .monospace(),
            );
        }

        MainBlock::Comment(text) => {
            ui.add(
                egui::TextEdit::singleline(text)
                    .desired_width(f32::INFINITY)
                    .hint_text("Comment text…"),
            );
        }

        MainBlock::RawCode(text) => {
            ui.add(
                egui::TextEdit::multiline(text)
                    .code_editor()
                    .desired_rows(3)
                    .desired_width(f32::INFINITY)
                    .hint_text("Raw C code…"),
            );
        }

        MainBlock::BlankLine => {
            ui.label(RichText::new("(blank line)").color(Color32::GRAY).small());
        }
    }
}

fn block_label_color(block: &MainBlock) -> Color32 {
    match block {
        MainBlock::VarDecl { .. } => Color32::from_rgb(100, 180, 255),
        MainBlock::FunctionCall { .. } => Color32::from_rgb(100, 220, 130),
        MainBlock::Comment(_) => Color32::from_rgb(180, 180, 100),
        MainBlock::RawCode(_) => Color32::from_rgb(200, 130, 100),
        MainBlock::BlankLine => Color32::GRAY,
    }
}

fn block_preview(block: &MainBlock) -> String {
    block.to_c().trim_start_matches('\t').to_string()
}

fn collect_project_funcs(project: &Project) -> Vec<String> {
    let src_dir = project.root.join("src");
    let Ok(entries) = std::fs::read_dir(&src_dir) else {
        return Vec::new();
    };
    let mut funcs = Vec::new();
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("c")
                && e.file_name() != "main.c"
        })
        .map(|e| e.path())
        .collect();
    paths.sort();
    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for f in extract_function_implementations(&content) {
                funcs.push(f.name);
            }
        }
    }
    funcs.sort();
    funcs
}
