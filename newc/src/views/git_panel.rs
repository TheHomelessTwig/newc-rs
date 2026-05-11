use iced::widget::{button, column, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length};
use newc_core::{git, project::Project};

use crate::state::{AppState, Message, View};

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("◆ Git — {}", project.name))
            .size(18)
            .color(Color::from_rgb(0.663, 0.863, 0.463)),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let is_repo = git::is_repo(&project.root);

    if !is_repo {
        return column![
            header,
            text("No git repository found.").color(Color::from_rgb(1.0, 0.847, 0.4)),
            button(text("Init repository")).on_press(Message::None),
        ]
        .spacing(10)
        .padding(16)
        .into();
    }

    // Branch row
    let branches = git::branches(&project.root);
    let cur_branch = git::current_branch(&project.root);
    let mut branch_btns: Vec<Element<Message>> = vec![
        text(format!("⎇ {}", cur_branch)).color(Color::from_rgb(0.314, 0.784, 0.471)).into(),
    ];
    for b in &branches {
        let b = b.clone();
        let _b2 = b.clone();
        if b != cur_branch {
            branch_btns.push(
                button(text(b.clone()).size(11)).on_press(Message::GitCheckout(b)).into(),
            );
        }
    }
    let branch_row = row(branch_btns).spacing(4).align_y(iced::Alignment::Center);

    let new_branch_row = row![
        text_input("New branch…", &state.git_new_branch)
            .on_input(Message::GitNewBranch)
            .width(180),
        button(text("Create").size(12)).on_press(Message::GitCreateBranch),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // Status
    let status_block: Element<Message> = if let Some(status) = git::status(&project.root) {
        column![
            row![
                text("Branch:").color(Color::WHITE),
                text(status.branch.clone()).color(Color::from_rgb(0.314, 0.784, 0.471)).font(iced::Font::MONOSPACE),
                text(format!("staged: {}  unstaged: {}  untracked: {}",
                    status.staged, status.unstaged, status.untracked))
                    .size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                Space::new().width(Length::Fill),
                button(text("Pull")).on_press(Message::GitPull),
                button(text("Push")).on_press(Message::GitPush),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(4)
        .padding(8)
        .into()
    } else {
        text("Unable to get git status").color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    };

    // Staged / unstaged files
    let changed = git::changed_files(&project.root);
    let (staged_files, unstaged_files): (Vec<_>, Vec<_>) = changed.iter().partition(|f| f.staged);

    let staged_section: Element<Message> = if staged_files.is_empty() {
        text("Nothing staged.").size(12).color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    } else {
        let rows: Vec<Element<Message>> = staged_files.iter().map(|f| {
            let path_str = f.path.clone();
            row![
                text(f.path.clone()).size(12).font(iced::Font::MONOSPACE).color(Color::from_rgb(0.663, 0.863, 0.463)),
                button(text("Unstage").size(10)).on_press(Message::GitUnstage(path_str)),
            ]
            .spacing(8)
            .into()
        }).collect();
        column(rows).spacing(2).into()
    };

    let unstaged_section: Element<Message> = if unstaged_files.is_empty() {
        text("No changes.").size(12).color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    } else {
        let rows: Vec<Element<Message>> = unstaged_files.iter().map(|f| {
            let path_str = f.path.clone();
            row![
                text(f.path.clone()).size(12).font(iced::Font::MONOSPACE).color(Color::from_rgb(1.0, 0.847, 0.4)),
                button(text("Stage").size(10)).on_press(Message::GitStage(path_str)),
            ]
            .spacing(8)
            .into()
        }).collect();
        column(rows).spacing(2).into()
    };

    // Commit
    let commit_row = row![
        text_input("Commit message…", &state.git_commit_msg)
            .on_input(Message::GitCommitMsg)
            .on_submit(Message::GitCommit)
            .width(320),
        button(text("Commit")).on_press(Message::GitCommit),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Log
    let log_entries = git::log(&project.root, 20);
    let log_rows: Vec<Element<Message>> = log_entries.iter().map(|entry| {
        row![
            text(entry.hash.clone()).size(11).font(iced::Font::MONOSPACE)
                .color(Color::from_rgb(1.0, 0.847, 0.4)).width(70),
            text(entry.author.clone()).size(11).width(120).color(Color::from_rgb(0.5, 0.5, 0.5)),
            text(entry.message.clone()).size(11),
        ]
        .spacing(8)
        .into()
    }).collect();

    column![
        header,
        branch_row,
        new_branch_row,
        status_block,
        text("Staged").size(13).color(Color::from_rgb(0.471, 0.863, 0.910)),
        staged_section,
        text("Unstaged").size(13).color(Color::from_rgb(0.471, 0.863, 0.910)),
        unstaged_section,
        commit_row,
        text("Recent commits").size(13).color(Color::from_rgb(0.471, 0.863, 0.910)),
        scrollable(column(log_rows).spacing(2)).height(200),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
