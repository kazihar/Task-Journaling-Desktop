use dioxus::prelude::*;

use crate::{
    types::{JournalProps, Screen},
    utils::delete_journal,
    CURRENT_SCREEN,
};
const DELETE: Asset = asset!("/assets/delete.png");

#[component]
pub fn JournalComponent(journal: JournalProps) -> Element {
    let title = journal.title.unwrap_or("Untitled".to_string());
    let body = journal.body.unwrap_or("".to_string());
    let tags = journal.tags;

    let handle_delete = move |_ev| {
        let id = journal.id.clone();
        spawn(async move {
            delete_journal(id.clone()).await;
            *CURRENT_SCREEN.write() = Screen::MainMenu;
        });
    };

    rsx!(
        div {
            class: "journal-container",
            img {
                style:"cursor: pointer;",
                src: DELETE,
                width: "24",
                height: "24",
                onclick: handle_delete
            }
            h2 { style:"text-decoration:underline;", "{title}" }
            div {
                style:"display: flex; flex-direction: row; justify-content: center; align-items: center; gap: 4px; flex: 1",
                h5 { "Tags : "}
                for tag in tags {
                    p {
                        style: "background-color: cyan; border: 1px solid black; padding: 2px; border-radius: 4px",
                        " {tag} "
                    }
                }
            }
            pre {"{body}"}
        }
    )
}
