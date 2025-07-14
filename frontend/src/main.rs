use dioxus::prelude::*;

use crate::{
    pages::{Create, MainMenu, Sysinfo},
    types::Screen,
};

pub mod components;
pub mod pages;
pub mod types;
pub mod utils;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

pub static CURRENT_SCREEN: GlobalSignal<Screen> = Signal::global(|| Screen::MainMenu);

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        match *CURRENT_SCREEN.read() {
            Screen::MainMenu => {
                rsx!{
                    MainMenu {}
                }
            }
            Screen::Create => {
                rsx!{
                    Create { }
                }
            }
            Screen::SysInfo => {
                rsx!{
                    Sysinfo { }
                }
            }
        }
    }
}
