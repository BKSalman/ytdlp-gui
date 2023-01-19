#![cfg_attr(not(debug_assertion), windows_subsystem = "windows")]

use std::fs;

use iced::{window, Application, Settings};
use log::warn;
use ytdlp_gui::{Config, YtGUI};

fn main() -> iced::Result {
    let config_dir = dirs::config_dir()
        .expect("config directory")
        .join("ytdlp-gui/");

    fs::create_dir_all(&config_dir).expect("create config dir");

    let config_file = match fs::read_to_string(config_dir.join("config.toml")) {
        Ok(file) => file,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                warn!("Config file not found, creating a default config file...");
                let new_config = toml::to_string(&Config::default()).expect("Config to string");
                fs::write(config_dir.join("config.toml"), &new_config)
                    .expect("create new config file");

                new_config
            }
            _ => panic!("{e}"),
        },
    };

    let config = toml::from_str::<Config>(&config_file).expect("Deserialize config file");

    let settings = Settings {
        id: Some("ytdlp-gui".to_string()),
        window: window::Settings {
            size: (600, 275),
            resizable: false,
            ..Default::default()
        },
        exit_on_close_request: false,
        flags: config,
        ..Default::default()
    };

    YtGUI::run(settings)
}
