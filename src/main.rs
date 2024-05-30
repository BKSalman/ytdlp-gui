#![cfg_attr(not(debug_assertion), windows_subsystem = "windows")]

use iced::{window, Application, Settings};
use ytdlp_gui::{git_hash, logging, Config, YtGUI};

fn main() -> iced::Result {
    let mut args = std::env::args();

    if let Some(arg) = args.nth(1) {
        if arg == "--help" || arg == "-h" {
            println!("Usage: ytdlp-gui <OPTIONS>\n");
            println!("Options:");
            println!("-h, --help     Print help");
            println!("-V, --version  Print version");
            std::process::exit(0);
        } else if arg == "--version" || arg == "-V" {
            let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
            let git_hash = git_hash!();
            println!("version: {version}");
            println!("git hash: {git_hash}");
            std::process::exit(0);
        } else {
            println!("Invalid option/argument");
            std::process::exit(1);
        }
    }

    logging();

    let config_dir = dirs::config_dir()
        .expect("config directory")
        .join("ytdlp-gui/");

    std::fs::create_dir_all(&config_dir).expect("create config dir");

    let config_file = match std::fs::read_to_string(config_dir.join("config.toml")) {
        Ok(file) => file,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                tracing::warn!("Config file not found, creating a default config file...");
                let new_config = toml::to_string(&Config::default()).expect("Config to string");
                std::fs::write(config_dir.join("config.toml"), &new_config)
                    .expect("create new config file");

                new_config
            }
            _ => panic!("{e}"),
        },
    };

    let config = toml::from_str::<Config>(&config_file).unwrap_or_else(|e| {
        tracing::error!("failed to parse config: {e:#?}");
        let config = Config::default();
        tracing::warn!("falling back to default configs: {config:#?}");
        config
    });

    let settings = Settings {
        id: Some(String::from("ytdlp-gui")),
        window: window::Settings {
            size: iced::Size::new(600., 360.),
            resizable: true,
            exit_on_close_request: false,
            ..Default::default()
        },
        flags: config,
        ..Default::default()
    };

    YtGUI::run(settings)
}
