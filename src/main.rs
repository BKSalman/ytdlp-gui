#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    Point,
    window::{self, Position},
};
use ytdlp_gui::{
    Config, Flags, YtGUI, git_hash, logging, theme::ytdlp_gui_theme, update::check_for_update,
};

fn main() -> iced::Result {
    let mut args = std::env::args();

    let mut url = None;
    if let Some(arg) = args.nth(1) {
        if arg == "--help" || arg == "-h" {
            println!("Usage: ytdlp-gui <OPTIONS>\n");
            println!("Options:");
            println!("-h, --help     Print help");
            println!("-V, --version  Print version");
            println!(
                "-u, --url      Starts the application with the provided URL as the download URL"
            );
            std::process::exit(0);
        } else if arg == "--version" || arg == "-V" {
            let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
            let git_hash = git_hash!();
            println!("version: {version}");
            println!("git hash: {git_hash}");
            std::process::exit(0);
        } else if arg == "--url" || arg == "-u" {
            url = std::env::args().nth(2);
        } else {
            println!("Invalid option/argument");
            std::process::exit(1);
        }
    }

    logging();

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    ytdlp_gui::i18n::init(&requested_languages);

    let config_dir = dirs::config_dir()
        .expect("config directory")
        .join("ytdlp-gui/");

    std::fs::create_dir_all(&config_dir).expect("create config dir");

    let config = match std::fs::read_to_string(config_dir.join("config.toml")) {
        Ok(config_str) => toml::from_str::<Config>(&config_str).unwrap_or_else(|e| {
            tracing::error!("failed to parse config: {e:#?}");
            let config = Config::default();
            tracing::warn!("falling back to default configs: {config:#?}");
            config
        }),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                let config = Config::default();
                tracing::warn!(
                    "Config file not found, falling back to default configs: {config:#?}"
                );
                config
            }
            _ => panic!("{e}"),
        },
    };

    let position = if config.save_window_position {
        if let Some(window_pos) = &config.window_position {
            Position::Specific(Point::new(window_pos.x, window_pos.y))
        } else {
            Position::default()
        }
    } else {
        Position::default()
    };

    let flags = Flags { url, config };

    let window_size = flags
        .config
        .window_size
        .map(|s| iced::Size::new(s.width, s.height))
        .unwrap_or(iced::Size::new(758., 425.));

    iced::application(
        move || {
            let flags = flags.clone();
            let (sender, receiver) = iced::futures::channel::mpsc::unbounded();
            (
                YtGUI::new(flags, sender),
                iced::Task::batch([
                    iced::Task::stream(receiver),
                    iced::Task::perform(check_for_update(), ytdlp_gui::Message::UpdateCheck),
                ]),
            )
        },
        YtGUI::update,
        YtGUI::view,
    )
    .title("Youtube Downloader")
    .window(window::Settings {
        size: window_size,
        resizable: true,
        exit_on_close_request: false,
        position,
        ..Default::default()
    })
    .subscription(YtGUI::subscription)
    .font(iced_fonts::REQUIRED_FONT_BYTES)
    .theme(ytdlp_gui_theme)
    .run()
}
