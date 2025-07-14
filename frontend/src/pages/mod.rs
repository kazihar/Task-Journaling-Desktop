use std::{fs::File, io::Write};

use dioxus::prelude::*;
use rfd::FileDialog;

use crate::{
    components::JournalComponent,
    types::{JournalProps, Screen, SystemInfo},
    utils::{create_journal, export, remove_from_vec},
    CURRENT_SCREEN,
};

const REFRESH: Asset = asset!("/assets/refresh.png");

#[component]
pub fn MainMenu() -> Element {
    let mut query = use_signal(|| "".to_string());

    let entries = use_resource(move || async move {
        let query = query.read().clone();

        if query.is_empty() {
            reqwest::get("http://127.0.0.1:7000/entries")
                .await
                .unwrap()
                .json::<Vec<JournalProps>>()
                .await
                .unwrap()
        } else {
            reqwest::get(format!("http://127.0.0.1:7000/entries?tag={query}"))
                .await
                .unwrap()
                .json::<Vec<JournalProps>>()
                .await
                .unwrap()
        }
    });

    let refresh = move |_evt| entries.clone().restart();

    use_effect(move || {
        let _ = CURRENT_SCREEN();
        entries.clone().restart();
    });

    let goto_create_page = move |_evt| {
        *CURRENT_SCREEN.write() = Screen::Create;
    };

    let goto_sysinfo_page = move |_evt| {
        *CURRENT_SCREEN.write() = Screen::SysInfo;
    };

    let export_as_file = move |_evt| {
        spawn(async move {
            match export().await {
                Ok(response) => {
                    let bytes = response.bytes().await.expect("Failed to read bytes");
                    if let Some(path) = FileDialog::new().set_file_name("journal.md").save_file() {
                        let mut file = File::create(path).expect("Failed to create file");
                        file.write_all(&bytes).expect("Failed to write to file");
                    }
                }
                Err(err) => eprintln!("{}", err.to_string()),
            };
        });
    };

    rsx! {
        div {
            class: "main-menu",
            div {
                class: "button-container-parent",
                div {
                    class: "button-container",
                    input {
                        class:"input-field",
                        value: query,
                        oninput: move |e| query.set(e.value()),
                        placeholder: "Search by tag"
                    }
                    button {
                        class:"refresh-button",
                        onclick: refresh,
                        img {
                            src: REFRESH,
                            alt: "icon",
                        }
                    }
                    button {
                        class:"export-button",
                        onclick: goto_sysinfo_page,
                        "System Info"
                    }
                    button {
                        class:"export-button",
                        disabled: entries.read().clone().unwrap_or_default().is_empty(),
                        onclick: export_as_file,
                        "Export"
                    }
                    button {
                        class:"create-button",
                        onclick: goto_create_page,
                        "Create"
                    }
                }
            }

            match entries.state().cloned() {
                UseResourceState::Ready => {
                    rsx!{
                        if entries.read().clone().unwrap_or_default().is_empty() {
                            if query.read().clone().is_empty() {
                                div {
                                    h2 { "Wow, So empty here" }
                                    h4 {"Click create to start"}
                                }

                            }else{
                                div {
                                    h2 {">__<"}
                                    h4 { "No journal with the tag {query.read().clone()} found!" }
                                }
                            }
                        }else{
                            for entry in entries.read().clone().unwrap_or_default() {
                                JournalComponent {key: entry.id, journal: entry}
                            }
                        }
                    }
                },
                _ => {
                    rsx!{
                        p { "Loading" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Create() -> Element {
    let mut title = use_signal(|| "".to_string());
    let mut body = use_signal(|| "".to_string());
    let mut tag_input = use_signal(|| "".to_string());
    let mut tags: Signal<Vec<String>> = use_signal(|| vec![]);

    let handle_add_tag = move |_evt| {
        let new_tag = tag_input.read().clone().trim().to_string();
        if !tags.clone().iter().any(|tag| tag.eq(&new_tag)) {
            tags.write().push(new_tag);
        }
        tag_input.set("".to_string());
    };

    let mut remove_tag = move |remove_tag: String| {
        let old_tags = tags.read().clone();
        *tags.write() = remove_from_vec(old_tags, remove_tag);
    };

    let handle_submit = move |_evt| {
        let new_title = title.read().clone();
        let new_body = body.read().clone();
        let new_tags = tags.read().clone();

        if !new_title.is_empty() && !new_body.is_empty() {
            spawn(async move {
                create_journal(new_body, new_title, new_tags).await;
                *CURRENT_SCREEN.write() = Screen::MainMenu;
            });
        }
    };

    let goto_main_menu = move |_evt| {
        *CURRENT_SCREEN.write() = Screen::MainMenu;
    };

    rsx! {
        div {
            class: "main-menu",
            div{
                h1 { "Create new Journal" }
                div {
                    input {
                        class: "input-field",
                        value: title,
                        oninput: move |e| title.set(e.value()),
                        placeholder: "Title"
                    }
                }
                br {}
                div {
                    textarea {
                        class: "input-field",
                        rows: "10",
                        cols: "50",
                        value: body,
                        oninput: move |e| body.set(e.value()),
                        placeholder: "Enter Content here"
                    }
                }
                br {}
                div {
                    span { "Tags :" }
                    div {
                        for tag in tags.read().clone() {
                            span {
                                class: "tag",
                                onclick:move |_| remove_tag(tag.clone()),
                                "{tag}, "
                            }
                        }
                    }
                    br {}
                    div {
                        style: "display: flex; flex-direction: row; gap: 8px;",
                        input {
                            class: "input-field",
                            value: tag_input,
                            oninput: move |e| tag_input.set(e.value()),
                            placeholder: "Enter Tag here"
                        }
                        button {
                            class: "export-button",
                            disabled: tag_input.read().is_empty(),
                            onclick: handle_add_tag,
                            "Add Tag"
                        }
                    }
                }
                div {
                    class: "button-container-parent",
                    div {
                        class: "button-container",
                        button {
                            class: "cancel-button",
                            onclick: goto_main_menu,
                            "Cancel"
                        }
                        button {
                            class:"create-button",
                            disabled: title.read().clone().is_empty() || body.read().clone().is_empty(),
                            onclick: handle_submit,
                            "Submit"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Sysinfo() -> Element {
    let infos = use_resource(|| async move {
        reqwest::get("http://127.0.0.1:7000/sysinfo")
            .await
            .unwrap()
            .json::<SystemInfo>()
            .await
            .unwrap()
    });

    let mut disk_usage = use_signal(|| vec![]);
    let mut current_user = use_signal(|| "".to_string());
    let mut cpu_processes = use_signal(|| vec![]);

    use_effect(move || {
        let info = infos.read().clone();

        if let Some(info) = info {
            let mut disk_result = vec![];
            for disk in &info.disk_usage {
                let total = disk.total_space / 1024 / 1024 / 1024;
                let available = disk.available_space / 1024 / 1024 / 1024;
                disk_result.push(format!(
                    "Mount Point:{} | {}GB/{}GB - ({:.2}%)",
                    disk.mount_point, available, total, disk.used_percentage
                ));
            }

            let mut processes = vec![];
            for process in &info.top_cpu_processes {
                processes.push(format!(
                    "PID:{} | Name: {} | CPU Usage: {}%",
                    process.pid, process.name, process.cpu_usage
                ));
            }

            *disk_usage.write() = disk_result;
            *cpu_processes.write() = processes;
            *current_user.write() = info.current_user.clone();
        }
    });

    let refresh = move |_evt| infos.clone().restart();

    let goto_main_menu = move |_evt| {
        *CURRENT_SCREEN.write() = Screen::MainMenu;
    };

    rsx! {
        div {
            class: "main-menu",
            div{
                div {
                    div {
                    class: "button-container",
                    button {
                        class: "cancel-button",
                        onclick: goto_main_menu,
                        "Back"
                    }
                    button {
                        class:"refresh-button",
                        onclick: refresh,
                        img {
                            src: REFRESH,
                            alt: "icon",
                        }
                    }
                }
                    h1 { "System Info" }
                    br {  }
                    br {  }
                    div {
                        h5 { "User: {current_user.read().clone()}" }
                    }
                    br {  }
                    h2 { "Disk usage" }
                    div {
                        for disk in disk_usage.read().clone() {
                            h5 {"{disk}"}
                        }
                    }
                    br {  }
                    h2 { "Top 5 CPU Processes" }
                    div {
                        for process in cpu_processes.read().clone() {
                            h5 {"{process}"}
                        }
                    }
                }
                div {

                }
            }
        }
    }
}
