use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Color, Element, Length};
use newc_core::project::Project;

use crate::state::{AppState, BuildState, Message, View};

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let build_running = matches!(state.build_state, BuildState::Running);
    let meta = &state.meta_draft;

    // ── Header ────────────────────────────────────────────────────────────────
    let header = row![
        button(text("← Home")).on_press(Message::Navigate(View::Home)),
        text(format!("◆ {}", project.name))
            .size(18)
            .color(Color::from_rgb(0.663, 0.863, 0.463)),
        text(project.root.display().to_string())
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
        Space::new().width(Length::Fill),
        button(text("Open in Editor")).on_press(Message::None),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Tool buttons ──────────────────────────────────────────────────────────
    let tools = row![
        button(text("Stats")).on_press(Message::Navigate(View::ProjectStats(project.clone()))),
        button(text("Notes")).on_press(Message::Navigate(View::ProjectNotes(project.clone()))),
        button(text("Git")).on_press(Message::Navigate(View::GitPanel(project.clone()))),
        button(text("History")).on_press(Message::Navigate(View::BuildHistory(project.clone()))),
        button(text("Health")).on_press(Message::Navigate(View::HealthDashboard(project.clone()))),
        button(text("Search")).on_press(Message::Navigate(View::ProjectSearch(project.clone()))),
        button(text("Usage")).on_press(Message::Navigate(View::UsageTracker(project.clone()))),
        button(text("Makefile")).on_press(Message::Navigate(View::MakefileEditor(project.clone()))),
        button(text("Export ZIP")).on_press(Message::None),
        button(text("Save Template")).on_press(Message::ShowSaveTemplate(true)),
        button(text("Compose main()")).on_press(Message::Navigate(View::MainBuilder(project.clone()))),
    ]
    .spacing(4)
    .wrap();

    // ── Metadata badges ───────────────────────────────────────────────────────
    let mut meta_row: Vec<Element<Message>> = vec![];
    if !meta.course.is_empty() {
        meta_row.push(text(format!("📚 {}", meta.course)).size(12)
            .color(Color::from_rgb(0.392, 0.706, 1.0)).into());
    }
    if !meta.assignment.is_empty() {
        meta_row.push(text(format!("📝 {}", meta.assignment)).size(12).into());
    }
    if let Some(days) = meta.days_until_due() {
        let (badge, color) = if days < 0 {
            (format!("⚠ Overdue by {} days", -days), Color::from_rgb(1.0, 0.314, 0.314))
        } else if days == 0 {
            ("⚠ Due today".into(), Color::from_rgb(1.0, 0.627, 0.0))
        } else if days <= 3 {
            (format!("⏰ Due in {days} days"), Color::from_rgb(1.0, 0.627, 0.0))
        } else {
            (format!("📅 Due in {days} days ({})", meta.due_date), Color::from_rgb(0.85, 0.85, 0.85))
        };
        meta_row.push(text(badge).size(12).color(color).into());
    }
    if let Some(m) = meta.marks_display() {
        meta_row.push(text(format!("🎯 {m}")).size(12).color(Color::from_rgb(0.392, 0.863, 0.392)).into());
    }
    meta_row.push(button(text("Edit Metadata").size(11)).on_press(Message::MetaShowEditor(true)).into());

    // ── Build targets ─────────────────────────────────────────────────────────
    let mut build_btns: Vec<Element<Message>> = vec![
        text("▶ Build:").color(Color::from_rgb(0.663, 0.863, 0.463)).into(),
    ];
    for target in ["all", "run", "debug", "release", "strict", "clean"] {
        let mut btn = button(text(target));
        if !build_running {
            btn = btn.on_press(Message::BuildStart(target.to_string()));
        }
        build_btns.push(btn.into());
    }
    if build_running {
        build_btns.push(text("⟳ building…").color(Color::from_rgb(1.0, 0.847, 0.4)).into());
    }
    let build_row = row(build_btns).spacing(4).align_y(iced::Alignment::Center);

    // ── Analysis tools ────────────────────────────────────────────────────────
    let analysis_row = row![
        text("⚙ Tools:").color(Color::from_rgb(0.671, 0.616, 0.949)),
        button(text("✓ Check")).on_press(Message::None),
        button(text("✂ Tidy")).on_press(Message::ShowTidyConfirm(true)),
        button(text("⟳ Sync All")).on_press(Message::None),
        button(text("↺ Refresh")).on_press(Message::RefreshProject),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // ── Module list ───────────────────────────────────────────────────────────
    let module_header = row![
        text("◈ Modules").size(14).color(Color::from_rgb(0.471, 0.863, 0.910)),
        button(text("+ Add Module").size(12))
            .on_press(Message::Navigate(View::AddModule { project: project.clone() })),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let module_list: Element<Message> = if project.modules.is_empty() {
        text("No modules found.").color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    } else {
        // Collect owned data
        struct ModRow { name: String, fn_count: usize }
        let mod_data: Vec<ModRow> = project.modules.iter().map(|m| ModRow {
            name: m.name.clone(),
            fn_count: m.function_count,
        }).collect();

        let col_header = row![
            text("Module").width(180).color(Color::from_rgb(0.671, 0.616, 0.949)),
            text("fns").width(50).color(Color::from_rgb(0.671, 0.616, 0.949)),
            Space::new().width(200),
        ]
        .spacing(4);

        let rows: Vec<Element<Message>> = mod_data.into_iter().map(|m| {
            let fn_color = match m.fn_count {
                0 => Color::from_rgb(0.447, 0.439, 0.447),
                1..=3 => Color::from_rgb(1.0, 0.847, 0.4),
                _ => Color::from_rgb(0.663, 0.863, 0.463),
            };
            let m_name = m.name.clone();
            let m_name2 = m.name.clone();
            let _m_name3 = m.name.clone();
            let proj = project.clone();
            let _proj2 = project.clone();
            row![
                text(format!("◆ {}", m.name)).width(180)
                    .color(Color::from_rgb(0.663, 0.863, 0.463)),
                text(m.fn_count.to_string()).width(50).color(fn_color),
                button(text("Edit").size(11))
                    .on_press(Message::Navigate(View::ModuleDetail { project: proj, module_name: m_name })),
                button(text("Sync").size(11)).on_press(Message::None),
                button(text("✗ Remove").size(11))
                    .on_press(Message::RemoveModule(m_name2)),
            ]
            .spacing(4)
            .into()
        }).collect();

        column![col_header, column(rows).spacing(2)].spacing(4).into()
    };

    // ── Confirm remove modal ──────────────────────────────────────────────────
    if let Some((_, mod_name)) = &state.confirm_remove_module {
        let mod_name = mod_name.clone();
        return column![
            text(format!("Remove module '{mod_name}'? This deletes the .c and .h files."))
                .color(Color::from_rgb(1.0, 0.4, 0.4)),
            row![
                button(text("Remove")).on_press(Message::ConfirmRemoveModule),
                button(text("Cancel")).on_press(Message::CancelRemoveModule),
            ]
            .spacing(8),
        ]
        .spacing(12)
        .padding(24)
        .into();
    }

    column![
        header,
        tools,
        row(meta_row).spacing(8),
        build_row,
        analysis_row,
        module_header,
        scrollable(module_list).height(Length::Fill),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
