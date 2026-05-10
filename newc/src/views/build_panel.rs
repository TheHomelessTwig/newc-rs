use egui::{Color32, RichText, ScrollArea, Ui};
use newc_core::diag::Diagnostic;

use crate::build_runner::{BuildLine, LineKind};

/// Returns `Some((file, line_no))` when the user clicks a diagnostic line.
pub fn show(
    ui: &mut Ui,
    lines: &[BuildLine],
    diagnostics: &[Diagnostic],
    auto_scroll: &mut bool,
) -> Option<(String, usize)> {
    let mut clicked: Option<(String, usize)> = None;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Build Output").strong());
        ui.checkbox(auto_scroll, "Auto-scroll");
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
                    // Check if this line matches a diagnostic — if so, make it clickable
                    let diag_match = diagnostics.iter().find(|d| {
                        let prefix = format!("{}:{}", d.file, d.line);
                        line.text.starts_with(&prefix)
                            || line.text.starts_with(&format!("./{prefix}"))
                    });
                    if let Some(d) = diag_match {
                        let resp = ui.add(
                            egui::Label::new(
                                RichText::new(&line.text)
                                    .color(color)
                                    .monospace()
                                    .underline(),
                            )
                            .sense(egui::Sense::click()),
                        );
                        if resp.clicked() {
                            clicked = Some((d.file.clone(), d.line));
                        }
                        resp.on_hover_text("Click to navigate to module");
                    } else {
                        ui.label(RichText::new(&line.text).color(color).monospace());
                    }
                }
            }
        });

    clicked
}
