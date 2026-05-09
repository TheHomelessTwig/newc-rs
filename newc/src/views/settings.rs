use egui::Ui;
use newc_core::config::AppConfig;

pub enum SettingsAction {
    None,
    Save(AppConfig),
}

pub fn show(ui: &mut Ui, config: &mut AppConfig) -> SettingsAction {
    let mut action = SettingsAction::None;

    ui.heading("Settings");
    ui.separator();

    egui::Grid::new("settings_grid")
        .num_columns(2)
        .min_col_width(120.0)
        .show(ui, |ui| {
            ui.label("Terminal:");
            ui.text_edit_singleline(&mut config.terminal);
            ui.end_row();

            ui.label("Editor:");
            ui.text_edit_singleline(&mut config.editor);
            ui.end_row();

            ui.label("Theme:");
            egui::ComboBox::from_id_salt("theme_combo")
                .selected_text(&config.theme)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut config.theme, "dark".to_string(), "Dark");
                    ui.selectable_value(&mut config.theme, "light".to_string(), "Light");
                });
            ui.end_row();

            ui.label("clang-format style:");
            egui::ComboBox::from_id_salt("clang_style_combo")
                .selected_text(&config.clang_format_style)
                .show_ui(ui, |ui| {
                    for style in ["file", "LLVM", "Google", "Chromium", "GNU", "Microsoft"] {
                        ui.selectable_value(&mut config.clang_format_style, style.to_string(), style);
                    }
                });
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.label("Scan directories (one per line, ~ supported):");
    let mut dirs_text = config.scan_dirs.join("\n");
    let resp = ui.add(
        egui::TextEdit::multiline(&mut dirs_text)
            .desired_rows(4)
            .desired_width(f32::INFINITY),
    );
    if resp.changed() {
        config.scan_dirs = dirs_text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
    }

    ui.add_space(8.0);
    if ui.button("Save").clicked() {
        action = SettingsAction::Save(config.clone());
    }

    ui.add_space(12.0);
    ui.separator();
    ui.label(egui::RichText::new("Terminal + editor are used by the 'Open in Editor' button on project detail view.").small().color(egui::Color32::GRAY));
    ui.label(egui::RichText::new("Scan dirs are searched on startup for existing newc projects (max depth 3).").small().color(egui::Color32::GRAY));

    action
}
