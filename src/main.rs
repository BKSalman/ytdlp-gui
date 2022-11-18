#![cfg_attr(not(debug_assertion), windows_subsystem = "windows")]

pub mod lib;

use iced::{Settings, window};
use ytdlp_gui::YtGUI;

fn main() -> iced::Result {
    let settings = Settings {
        id: Some("ytdlp-gui".to_string()),
        window: window::Settings {
            size: (600, 275),
            resizable: false,
            ..Default::default()
        },
        exit_on_close_request: false,
        ..Default::default()
    };

    YtGUI::run(settings)
}
