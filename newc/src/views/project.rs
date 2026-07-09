//! Project detail screen — metadata badges, build controls, code tools, and module table.

use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Background, Border, Element, Length};
use newc_core::project::Project;

use crate::state::{AppState, BuildState, Message, View};
use crate::theme as th;

/// Renders the project detail screen for the given project.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let build_running = matches!(state.build_state, BuildState::Running);
    let meta = &state.meta_draft;

    // ── Confirm remove modal ──────────────────────────────────────────────────
    if let Some((_, mod_name)) = &state.confirm_remove_module {
        let mod_name = mod_name.clone();
        return container(
            column![
                text(format!("Remove module '{mod_name}'?"))
                    .size(15)
                    .color(th::color::accent()),
                th::hint_text("This permanently deletes the .c and .h files."),
                row![
                    button(text("Remove").size(12))
                        .on_press(Message::ConfirmRemoveModule)
                        .style(th::btn_danger),
                    button(text("Cancel").size(12))
                        .on_press(Message::CancelRemoveModule)
                        .style(th::btn_secondary),
                ]
                .spacing(8),
            ]
            .spacing(12)
            .padding(20),
        )
        .style(th::card_style)
        .padding(4)
        .into();
    }

    // ── Header ────────────────────────────────────────────────────────────────
    let header = row![
        button(text("← Home").size(12))
            .on_press(Message::Navigate(View::Home))
            .style(th::btn_ghost),
        text(format!("◆ {}", project.name))
            .size(18)
            .color(th::color::green()),
        text(project.root.display().to_string())
            .size(11)
            .color(th::color::text_hint()),
        Space::new().width(Length::Fill),
        button(text("Open in Editor").size(12))
            .on_press(Message::OpenInEditor)
            .style(th::btn_secondary),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Metadata badges ───────────────────────────────────────────────────────
    let mut meta_row: Vec<Element<Message>> = vec![];
    if !meta.course.is_empty() {
        meta_row.push(
            container(
                text(format!("📚 {}", meta.course))
                    .size(11)
                    .color(th::color::cyan()),
            )
            .padding([2, 6])
            .style(th::section_style)
            .into(),
        );
    }
    if !meta.assignment.is_empty() {
        meta_row.push(
            container(
                text(format!("📝 {}", meta.assignment))
                    .size(11)
                    .color(th::color::text()),
            )
            .padding([2, 6])
            .style(th::section_style)
            .into(),
        );
    }
    if let Some(days) = meta.days_until_due() {
        let (badge, color) = if days < 0 {
            (format!("⚠ Overdue by {} days", -days), th::color::accent())
        } else if days == 0 {
            ("⚠ Due today".into(), th::color::yellow())
        } else if days <= 3 {
            (format!("⏰ Due in {days} days"), th::color::yellow())
        } else {
            (
                format!("📅 Due in {days} days ({})", meta.due_date),
                th::color::text_dim(),
            )
        };
        meta_row.push(
            container(text(badge).size(11).color(color))
                .padding([2, 6])
                .style(th::section_style)
                .into(),
        );
    }
    if let Some(m) = meta.marks_display() {
        meta_row.push(
            container(text(format!("🎯 {m}")).size(11).color(th::color::green()))
                .padding([2, 6])
                .style(th::section_style)
                .into(),
        );
    }
    meta_row.push(
        button(text("Edit Metadata").size(11))
            .on_press(Message::MetaShowEditor(true))
            .style(th::btn_ghost)
            .into(),
    );

    // ── Navigate section ──────────────────────────────────────────────────────
    let nav_section = container(
        column![
            th::section_title("Navigate"),
            row![
                button(text("Stats").size(12))
                    .on_press(Message::Navigate(View::ProjectStats(project.clone())))
                    .style(th::btn_secondary),
                button(text("Notes").size(12))
                    .on_press(Message::Navigate(View::ProjectNotes(project.clone())))
                    .style(th::btn_secondary),
                button(text("Git").size(12))
                    .on_press(Message::Navigate(View::GitPanel(project.clone())))
                    .style(th::btn_secondary),
                button(text("History").size(12))
                    .on_press(Message::Navigate(View::BuildHistory(project.clone())))
                    .style(th::btn_secondary),
                button(text("Health").size(12))
                    .on_press(Message::Navigate(View::HealthDashboard(project.clone())))
                    .style(th::btn_secondary),
                button(text("Search").size(12))
                    .on_press(Message::Navigate(View::ProjectSearch(project.clone())))
                    .style(th::btn_secondary),
                button(text("Usage").size(12))
                    .on_press(Message::Navigate(View::UsageTracker(project.clone())))
                    .style(th::btn_secondary),
                button(text(project.build_system.build_file_name()).size(12))
                    .on_press(Message::Navigate(View::MakefileEditor(project.clone())))
                    .style(th::btn_secondary),
            ]
            .spacing(4)
            .wrap(),
        ]
        .spacing(6)
        .padding(10),
    )
    .style(th::section_style);

    // ── Build section ─────────────────────────────────────────────────────────
    let mut build_btns: Vec<Element<Message>> = vec![];
    for target in ["all", "run", "debug", "release", "strict", "clean"] {
        let mut btn = button(text(target).size(12)).style(th::btn_secondary);
        if !build_running {
            btn = btn.on_press(Message::BuildStart(target.to_string()));
        }
        build_btns.push(btn.into());
    }
    if build_running {
        build_btns.push(
            text("⟳ building…")
                .size(12)
                .color(th::color::yellow())
                .into(),
        );
    }

    let build_section = container(
        column![
            th::section_title("▶ Build"),
            row(build_btns).spacing(4).align_y(iced::Alignment::Center),
        ]
        .spacing(6)
        .padding(10),
    )
    .style(th::section_style);

    // ── Code section ──────────────────────────────────────────────────────────
    let code_section = container(
        column![
            th::section_title("Code"),
            row![
                button(text("Compose main()").size(12))
                    .on_press(Message::Navigate(View::MainBuilder(project.clone())))
                    .style(th::btn_secondary),
                button(text("Call Graph").size(12))
                    .on_press(Message::Navigate(View::CallGraph(project.clone())))
                    .style(th::btn_secondary),
                button(text("Dep Graph").size(12))
                    .on_press(Message::Navigate(View::DependencyGraph(project.clone())))
                    .style(th::btn_secondary),
                button(text("Flowchart").size(12))
                    .on_press(Message::Navigate(View::FlowChart(project.clone())))
                    .style(th::btn_secondary),
            ]
            .spacing(4),
        ]
        .spacing(6)
        .padding(10),
    )
    .style(th::section_style);

    // ── Manage section ────────────────────────────────────────────────────────
    let manage_section = container(
        column![
            th::section_title("Manage"),
            row![
                button(text("✓ Check").size(12))
                    .on_press(Message::RunCheck)
                    .style(th::btn_secondary),
                button(text("✂ Tidy").size(12))
                    .on_press(Message::ShowTidyConfirm(true))
                    .style(th::btn_secondary),
                button(text("⟳ Sync All").size(12))
                    .on_press(Message::SyncAll)
                    .style(th::btn_secondary),
                button(text("↺ Refresh").size(12))
                    .on_press(Message::RefreshProject)
                    .style(th::btn_secondary),
                button(text("Export ZIP").size(12))
                    .on_press(Message::ExportZip)
                    .style(th::btn_secondary),
                button(text("compile_commands.json").size(12))
                    .on_press(Message::ExportCompileCommands)
                    .style(th::btn_secondary),
                button(text("Save Template").size(12))
                    .on_press(Message::ShowSaveTemplate(true))
                    .style(th::btn_secondary),
                button(text("Report").size(12))
                    .on_press(Message::GenerateReport)
                    .style(th::btn_secondary),
            ]
            .spacing(4)
            .wrap(),
            row![
                text("Pack submission:")
                    .size(12)
                    .color(th::color::text_dim()),
                text_input("student name", &state.submission_student_input)
                    .on_input(Message::SubmissionStudentInput)
                    .width(160),
                text_input("assignment #", &state.submission_assignment_input)
                    .on_input(Message::SubmissionAssignmentInput)
                    .width(100),
                button(text("Pack").size(12))
                    .on_press(Message::PackSubmission)
                    .style(th::btn_secondary),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(6)
        .padding(10),
    )
    .style(th::section_style);

    // ── Module table ──────────────────────────────────────────────────────────
    let module_section = {
        let col_header = container(
            row![
                text("Module")
                    .size(11)
                    .width(180)
                    .color(th::color::text_dim()),
                text("fns").size(11).width(40).color(th::color::text_dim()),
                text("size").size(11).width(52).color(th::color::text_dim()),
                Space::new().width(Length::Fill),
            ]
            .spacing(4)
            .padding([4, 8]),
        )
        .style(|_| iced::widget::container::Style {
            background: Some(Background::Color(th::color::bg_deep())),
            border: Border {
                color: th::color::border_dim(),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        let module_rows: Element<Message> = if project.modules.is_empty() {
            th::hint_text("No modules found.").into()
        } else {
            struct ModRow {
                name: String,
                fn_count: usize,
                src_bytes: u64,
            }
            let mod_data: Vec<ModRow> = project
                .modules
                .iter()
                .map(|m| ModRow {
                    name: m.name.clone(),
                    fn_count: m.function_count,
                    src_bytes: std::fs::metadata(&m.source).map(|md| md.len()).unwrap_or(0),
                })
                .collect();

            let rows: Vec<Element<Message>> = mod_data
                .into_iter()
                .enumerate()
                .map(|(i, m)| {
                    let fn_color = match m.fn_count {
                        0 => th::color::text_hint(),
                        1..=3 => th::color::yellow(),
                        _ => th::color::green(),
                    };
                    let m_name = m.name.clone();
                    let m_name2 = m.name.clone();
                    let proj = project.clone();
                    let row_style = if i % 2 == 0 {
                        th::stripe_even
                    } else {
                        th::stripe_odd
                    };
                    container(
                        row![
                            text(format!("◆ {}", m.name))
                                .size(12)
                                .width(180)
                                .color(th::color::green()),
                            text(m.fn_count.to_string())
                                .size(12)
                                .width(40)
                                .color(fn_color),
                            text(fmt_bytes(m.src_bytes))
                                .size(11)
                                .width(52)
                                .color(th::color::text_dim()),
                            Space::new().width(Length::Fill),
                            button(text("Edit").size(11))
                                .on_press(Message::Navigate(View::ModuleDetail {
                                    project: proj,
                                    module_name: m_name
                                }))
                                .style(th::btn_ghost),
                            button(text("Sync").size(11))
                                .on_press(Message::SyncModule(m.name.clone()))
                                .style(th::btn_ghost),
                            button(text("✗").size(11))
                                .on_press(Message::RemoveModule(m_name2))
                                .style(th::btn_danger),
                        ]
                        .spacing(4)
                        .align_y(iced::Alignment::Center)
                        .padding([4, 8]),
                    )
                    .width(Length::Fill)
                    .style(row_style)
                    .into()
                })
                .collect();

            column(rows).spacing(0).into()
        };

        column![
            row![
                th::section_title("◈ Modules"),
                Space::new().width(Length::Fill),
                button(text("+ Add Module").size(11))
                    .on_press(Message::Navigate(View::AddModule {
                        project: project.clone()
                    }))
                    .style(th::btn_primary),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
            col_header,
            scrollable(module_rows).height(Length::Fill),
        ]
        .spacing(4)
    };

    column![
        header,
        th::separator(),
        row(meta_row).spacing(6).align_y(iced::Alignment::Center),
        row![nav_section, build_section, code_section, manage_section]
            .spacing(6)
            .wrap(),
        module_section,
    ]
    .spacing(8)
    .padding(12)
    .into()
}

fn fmt_bytes(b: u64) -> String {
    if b < 1024 {
        format!("{b}B")
    } else {
        format!("{:.1}K", b as f64 / 1024.0)
    }
}
