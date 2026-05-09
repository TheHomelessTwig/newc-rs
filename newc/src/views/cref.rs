use egui::{Color32, RichText, ScrollArea, SidePanel, CentralPanel, Ui};
use newc_core::cref::{self, CFunc};

#[derive(Default)]
pub struct CRefState {
    pub search: String,
    pub selected_header: Option<String>,
    pub selected_func: Option<&'static str>,
}

pub fn show(ui: &mut Ui, state: &mut CRefState) {
    ui.horizontal(|ui| {
        ui.heading("C Standard Library Reference");
        ui.separator();
        ui.text_edit_singleline(&mut state.search)
            .labelled_by(ui.label("Search:").id);
        if !state.search.is_empty() && ui.small_button("✕").clicked() {
            state.search.clear();
        }
    });
    ui.separator();

    let results: Vec<&CFunc> = if state.search.is_empty() {
        match &state.selected_header {
            Some(h) => cref::by_header(h),
            None => cref::all_functions().iter().collect(),
        }
    } else {
        cref::search(&state.search)
    };

    // Left: header list
    SidePanel::left("cref_headers")
        .min_width(130.0)
        .max_width(180.0)
        .show_inside(ui, |ui| {
            ui.label(RichText::new("Headers").strong());
            ui.separator();
            let all_sel = state.selected_header.is_none() && state.search.is_empty();
            if ui.selectable_label(all_sel, "All").clicked() {
                state.selected_header = None;
                state.search.clear();
            }
            for h in cref::headers() {
                let sel = state.selected_header.as_deref() == Some(h) && state.search.is_empty();
                let count = cref::by_header(h).len();
                if ui.selectable_label(sel, format!("<{h}> ({count})")).clicked() {
                    state.selected_header = Some(h.to_string());
                    state.search.clear();
                    state.selected_func = None;
                }
            }
        });

    // Middle: function list
    SidePanel::left("cref_list")
        .min_width(160.0)
        .max_width(220.0)
        .show_inside(ui, |ui| {
            ui.label(RichText::new("Functions").strong());
            ui.separator();
            ScrollArea::vertical().id_salt("cref_fn_list").show(ui, |ui| {
                for func in &results {
                    let sel = state.selected_func == Some(func.name);
                    ui.horizontal(|ui| {
                        if ui.selectable_label(sel, func.name).clicked() {
                            state.selected_func = Some(func.name);
                        }
                        ui.label(
                            RichText::new(format!("<{}>", func.header))
                                .small()
                                .color(Color32::GRAY),
                        );
                    });
                }
                if results.is_empty() {
                    ui.label(RichText::new("No results.").color(Color32::GRAY).small());
                }
            });
        });

    // Right: function detail
    CentralPanel::default().show_inside(ui, |ui| {
        let Some(name) = state.selected_func else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select a function").color(Color32::GRAY));
            });
            return;
        };
        let Some(func) = cref::all_functions().iter().find(|f| f.name == name) else {
            return;
        };

        ui.horizontal(|ui| {
            ui.heading(func.name);
            ui.label(
                RichText::new(format!("<{}>", func.header))
                    .color(Color32::from_rgb(100, 180, 255)),
            );
            if ui.small_button("Copy #include").clicked() {
                ui.ctx().copy_text(format!("#include <{}>", func.header));
            }
        });
        ui.separator();

        ui.label(RichText::new("Signature:").strong());
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(func.signature).monospace());
                if ui.small_button("Copy").clicked() {
                    ui.ctx().copy_text(func.signature.to_string());
                }
            });
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Description:").strong());
        ui.label(func.description);

        ui.add_space(8.0);
        ui.label(RichText::new("Returns:").strong());
        ui.label(func.returns);

        ui.add_space(8.0);
        ui.label(RichText::new("Example:").strong());
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(RichText::new(func.example).monospace().size(12.0))
                        .wrap_mode(egui::TextWrapMode::Wrap),
                );
                if ui.small_button("Copy").clicked() {
                    ui.ctx().copy_text(func.example.to_string());
                }
            });
        });
    });
}
