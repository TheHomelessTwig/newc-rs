//! Home screen — project list with workspace tabs, first-run onboarding card, and quick actions.

use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Background, Border, Element, Length};

use crate::state::{AppState, Message, View};
use crate::theme as th;

fn project_card_style(_: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => th::color::bg_raised(),
        _ => th::color::bg_card(),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: th::color::border_dim(),
            width: 1.0,
            radius: 6.0.into(),
        },
        text_color: th::color::text(),
        ..Default::default()
    }
}

/// Renders the home screen with the project list, workspace tabs, and optional onboarding.
pub fn view(state: &AppState) -> Element<'_, Message> {
    // ── Header ───────────────────────────────────────────────────────────────
    let header = row![
        th::heading("⌂ Projects"),
        Space::new().width(Length::Fill),
        button(text("+ New Project").size(12))
            .on_press(Message::Navigate(View::CreateProject))
            .style(th::btn_primary),
        button(text("Open Folder…").size(12))
            .on_press(Message::BrowseForProject)
            .style(th::btn_secondary),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Workspace tabs ────────────────────────────────────────────────────────
    let all_active = state.active_workspace.is_none() && !state.show_archived;
    let mut ws_row: Vec<Element<Message>> = vec![
        button(text("All").size(12))
            .on_press(Message::WorkspaceSelect(None))
            .style(if all_active {
                th::btn_nav_active
            } else {
                th::btn_nav_inactive
            })
            .into(),
    ];
    for ws in &state.config.workspaces {
        if ws.name == "__archived__" {
            continue;
        }
        let active = state.active_workspace.as_deref() == Some(ws.name.as_str());
        ws_row.push(
            button(text(ws.name.clone()).size(12))
                .on_press(Message::WorkspaceSelect(Some(ws.name.clone())))
                .style(if active {
                    th::btn_nav_active
                } else {
                    th::btn_nav_inactive
                })
                .into(),
        );
    }
    ws_row.push(
        button(text("Archived").size(12))
            .on_press(Message::ShowArchivedToggle)
            .style(if state.show_archived {
                th::btn_nav_active
            } else {
                th::btn_nav_inactive
            })
            .into(),
    );
    ws_row.push(
        button(text("+").size(12))
            .on_press(Message::WorkspaceNew)
            .style(th::btn_ghost)
            .into(),
    );
    let ws_tabs = row(ws_row).spacing(4).align_y(iced::Alignment::Center);

    // ── New workspace input ───────────────────────────────────────────────────
    let new_ws_row: Option<Element<Message>> = if state.show_new_workspace {
        Some(
            row![
                text_input("Workspace name…", &state.workspace_input)
                    .on_input(Message::WorkspaceInput)
                    .style(th::input_style)
                    .width(200),
                button(text("Create").size(12))
                    .on_press(Message::WorkspaceNew)
                    .style(th::btn_primary),
                button(text("Cancel").size(12))
                    .on_press(Message::WorkspaceCancelNew)
                    .style(th::btn_ghost),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into(),
        )
    } else {
        None
    };

    // ── First-run onboarding ──────────────────────────────────────────────────
    let onboard: Option<Element<Message>> = if state.is_first_run && state.known_projects.is_empty()
    {
        Some(
            container(
                column![
                    th::heading("Welcome to newc!"),
                    th::subheading(
                        "Scaffold, build, and manage C projects with reusable function libraries."
                    ),
                    th::section_title("Getting started"),
                    th::label_text("1. Create your first project with \"New Project\""),
                    th::label_text("2. Add modules and import functions from the Function Library"),
                    th::label_text("3. Use Compose main() to visually build your main.c"),
                    th::label_text("4. Browse the C Reference for stdlib documentation"),
                    button(text("Create first project").size(13))
                        .on_press(Message::Navigate(View::CreateProject))
                        .style(th::btn_primary),
                ]
                .spacing(8)
                .padding(16),
            )
            .style(th::card_style)
            .into(),
        )
    } else {
        None
    };

    // ── Project list ──────────────────────────────────────────────────────────
    let archived_paths: Vec<_> = state
        .config
        .workspaces
        .iter()
        .find(|w| w.name == "__archived__")
        .map(|w| w.paths.clone())
        .unwrap_or_default();

    let workspace_paths: Option<Vec<_>> = state.active_workspace.as_ref().and_then(|ws_name| {
        state
            .config
            .workspaces
            .iter()
            .find(|w| &w.name == ws_name)
            .map(|w| w.paths.clone())
    });

    let visible_projects: Vec<_> = state
        .known_projects
        .iter()
        .filter(|path| {
            let is_archived = archived_paths.contains(path);
            if state.show_archived {
                return is_archived;
            }
            if is_archived {
                return false;
            }
            match &workspace_paths {
                Some(ws_paths) => ws_paths.contains(path),
                None => true,
            }
        })
        .collect();

    struct ProjRow {
        name: String,
        path_str: String,
        missing: bool,
        path: std::path::PathBuf,
    }
    let proj_data: Vec<ProjRow> = visible_projects
        .iter()
        .map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            ProjRow {
                name,
                path_str: path.display().to_string(),
                missing: !path.exists(),
                path: (*path).clone(),
            }
        })
        .collect();

    let project_list: Element<Message> = if proj_data.is_empty() {
        container(
            column![
                th::hint_text(if state.known_projects.is_empty() {
                    "No projects yet. Create one or open an existing folder."
                } else {
                    "No projects in this workspace."
                }),
                button(text("+ New Project").size(12))
                    .on_press(Message::Navigate(View::CreateProject))
                    .style(th::btn_primary),
            ]
            .spacing(12)
            .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(40)
        .into()
    } else {
        let cards: Vec<Element<Message>> = proj_data
            .into_iter()
            .map(|p| {
                let name_color = if p.missing {
                    th::color::accent()
                } else {
                    th::color::green()
                };
                let label = if p.missing {
                    format!("⚠ {} (missing)", p.name)
                } else {
                    format!("◆ {}", p.name)
                };
                let path_clone = p.path.clone();
                let inner = column![
                    text(label).size(13).color(name_color),
                    text(p.path_str).size(11).color(th::color::text_hint()),
                ]
                .spacing(3)
                .padding([6, 4]);

                if p.missing {
                    container(inner)
                        .width(Length::Fill)
                        .padding(4)
                        .style(th::card_style)
                        .into()
                } else {
                    button(inner)
                        .on_press(Message::OpenProject(path_clone))
                        .padding(4)
                        .style(project_card_style)
                        .width(Length::Fill)
                        .into()
                }
            })
            .collect();

        scrollable(column(cards).spacing(6))
            .height(Length::Fill)
            .into()
    };

    // ── Assemble ──────────────────────────────────────────────────────────────
    let mut content = column![header, th::separator(), ws_tabs].spacing(0);
    if let Some(ws_input) = new_ws_row {
        content = content.push(ws_input);
    }
    if let Some(ob) = onboard {
        content = content.push(ob);
    }
    content = content.push(project_list);
    content.spacing(10).padding(16).into()
}
