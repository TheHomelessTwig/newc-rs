use iced::widget::{button, column, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length};

use crate::state::{AppState, Message, View};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut col = column![
        row![
            text("⌂ Projects").size(20),
            Space::new().width(Length::Fill),
            button(text("+ New Project"))
                .on_press(Message::Navigate(View::CreateProject)),
            button(text("Open Folder…")).on_press(Message::BrowseForProject),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(10)
    .padding(16);

    // First-run onboarding
    if state.is_first_run && state.known_projects.is_empty() {
        col = col.push(
            column![
                text("Welcome to newc!").size(16),
                text("newc helps you scaffold, build and manage C projects with reusable function libraries."),
                text("Getting started:").size(13),
                text("1. Create your first project with \"New Project\""),
                text("2. Add modules and import functions from the Function Library"),
                text("3. Use Compose main() to visually build your main.c"),
                text("4. Browse the C Reference for stdlib documentation"),
                button(text("Create first project")).on_press(Message::Navigate(View::CreateProject)),
            ]
            .spacing(6)
            .padding(12),
        );
    }

    // Workspace tabs
    let mut ws_row: Vec<Element<Message>> = vec![];
    let _all_sel = state.active_workspace.is_none() && !state.show_archived;
    ws_row.push(
        button(text("All").size(12))
            .on_press(Message::WorkspaceSelect(None))
            .into(),
    );
    for ws in &state.config.workspaces {
        if ws.name == "__archived__" { continue; }
        ws_row.push(
            button(text(ws.name.clone()).size(12))
                .on_press(Message::WorkspaceSelect(Some(ws.name.clone())))
                .into(),
        );
    }
    ws_row.push(
        button(text("Archived").size(12))
            .on_press(Message::ShowArchivedToggle)
            .into(),
    );
    ws_row.push(button(text("+").size(12)).on_press(Message::WorkspaceNew).into());
    col = col.push(row(ws_row).spacing(4).align_y(iced::Alignment::Center));

    // New workspace input
    if state.show_new_workspace {
        col = col.push(
            row![
                text_input("Workspace name…", &state.workspace_input)
                    .on_input(Message::WorkspaceInput)
                    .width(200),
                button(text("Create")).on_press(Message::WorkspaceNew),
                button(text("Cancel")).on_press(Message::WorkspaceCancelNew),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        );
    }

    if state.known_projects.is_empty() {
        col = col.push(
            text("No projects yet. Create one or open an existing folder.")
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        );
        return col.into();
    }

    // Filter projects
    let archived_paths: Vec<_> = state.config.workspaces.iter()
        .find(|w| w.name == "__archived__")
        .map(|w| w.paths.clone())
        .unwrap_or_default();

    let workspace_paths: Option<Vec<_>> = state.active_workspace.as_ref().and_then(|ws_name| {
        state.config.workspaces.iter().find(|w| &w.name == ws_name).map(|w| w.paths.clone())
    });

    let visible_projects: Vec<_> = state.known_projects.iter().filter(|path| {
        let is_archived = archived_paths.contains(path);
        if state.show_archived { return is_archived; }
        if is_archived { return false; }
        match &workspace_paths {
            Some(ws_paths) => ws_paths.contains(path),
            None => true,
        }
    }).collect();

    if visible_projects.is_empty() {
        col = col.push(text("No projects in this workspace.").color(Color::from_rgb(0.5, 0.5, 0.5)));
        return col.into();
    }

    // Project list — collect owned data first
    struct ProjRow { name: String, path_str: String, missing: bool, path: std::path::PathBuf }
    let proj_data: Vec<ProjRow> = visible_projects.iter().map(|path| {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        ProjRow {
            name,
            path_str: path.display().to_string(),
            missing: !path.exists(),
            path: (*path).clone(),
        }
    }).collect();

    let project_rows: Vec<Element<Message>> = proj_data.into_iter().map(|p| {
        let color = if p.missing { Color::from_rgb(1.0, 0.38, 0.533) } else { Color::from_rgb(0.663, 0.863, 0.463) };
        let label = if p.missing { format!("⚠ {} (missing)", p.name) } else { format!("⌂ {}", p.name) };
        let path_clone = p.path.clone();
        let mut btn = button(text(label).color(color));
        if !p.missing {
            btn = btn.on_press(Message::OpenProject(path_clone));
        }
        row![
            btn.width(200),
            text(p.path_str).size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    }).collect();

    col.push(scrollable(column(project_rows).spacing(4))).into()
}
