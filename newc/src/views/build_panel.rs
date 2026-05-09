use egui::{Color32, RichText, ScrollArea, Ui};

use crate::build_runner::{BuildLine, LineKind};

pub fn show(ui: &mut Ui, lines: &[BuildLine], auto_scroll: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Build Output").strong());
        ui.checkbox(auto_scroll, "Auto-scroll");
        if ui.small_button("Clear").clicked() {
            // Caller must handle clear — we signal via return value.
            // For simplicity: handled by the app layer checking cleared flag.
        }
    });
    ui.separator();

    ScrollArea::vertical()
        .id_salt("build_output")
        .stick_to_bottom(*auto_scroll)
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            for line in lines {
                let color = match line.kind {
                    LineKind::Stdout => Color32::LIGHT_GRAY,
                    LineKind::Stderr => Color32::from_rgb(255, 100, 100),
                    LineKind::Info => Color32::from_rgb(100, 180, 255),
                    LineKind::Done { exit_code: Some(0), .. } => Color32::from_rgb(100, 220, 100),
                    LineKind::Done { .. } => Color32::from_rgb(255, 80, 80),
                };
                if let LineKind::Done { exit_code, duration_ms } = line.kind {
                    let timing = format!("{:.1}s", duration_ms as f64 / 1000.0);
                    let msg = match exit_code {
                        Some(0) => format!("Build succeeded in {timing}."),
                        Some(c) => format!("Build failed (exit {c}) in {timing}."),
                        None => format!("Build terminated after {timing}."),
                    };
                    ui.label(RichText::new(msg).color(color).strong());
                } else if !line.text.is_empty() {
                    ui.label(RichText::new(&line.text).color(color).monospace());
                }
            }
        });
}
