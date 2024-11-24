#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

static MARIO_BG: &str = "./SMWCase.jpg";

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");

    LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Barely Console")
                    .with_always_on_top(false),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut is_active = use_signal(|| None::<bool>);

    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        h1 {
            onclick: move |_| {
                is_active.set(match is_active() {
                    None => Some(true),
                    Some(true) => Some(false),
                    Some(false) => Some(true),
                });
            },
            "Barely Console"
        }
        div { id: "preview",
            class: match *is_active.read() {
                None => "hidden",
                Some(true) => "active",
                Some(false) => "",
            },
            style: format!("background: url({}) no-repeat center center; background-size: contain", MARIO_BG),
        }
    }
}
