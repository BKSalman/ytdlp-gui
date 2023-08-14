use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

use chrono::Local;
use iced::{executor, widget::container};
use iced::{
    futures::channel::mpsc::UnboundedSender,
    widget::{button, checkbox, column, progress_bar, row, text, text_input},
    Application, Length, Subscription,
};
use iced_aw::Card;
use iced_native::subscription;

use native_dialog::FileDialog;
use serde::{Deserialize, Serialize};

pub mod command;
pub mod media_options;
pub mod progress;
pub mod theme;
pub mod widgets;

use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use widgets::{Modal, Tabs};

use crate::media_options::{playlist_options, Options};
use crate::media_options::{AudioFormat, AudioQuality, VideoFormat, VideoResolution};
use crate::progress::{bind, parse_progress, Progress};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const FONT_SIZE: u16 = 18;

const SPACING: u16 = 10;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    InputChanged(String),
    TogglePlaylist(bool),
    SelectedVideoFormat(VideoFormat),
    SelectedResolution(VideoResolution),
    SelectedAudioFormat(AudioFormat),
    SelectedAudioQuality(AudioQuality),
    SelectFolder,
    SelectFolderTextInput(String),
    SelectTab(usize),
    ProgressEvent(String),
    Ready(UnboundedSender<String>),
    Command(command::Message),
    IcedEvent(iced_native::Event),
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    bin_dir: Option<PathBuf>,
    download_folder: Option<PathBuf>,
    options: Options,
}

impl Config {
    fn update_config_file(&self) -> io::Result<()> {
        let current_config = toml::to_string(self).expect("config to string");
        let config_file = dirs::config_dir()
            .expect("config directory")
            .join("ytdlp-gui/config.toml");
        fs::write(config_file, &current_config)?;
        tracing::info!("Updated config file to {}", current_config);
        Ok(())
    }
}

pub struct YtGUI {
    download_link: String,
    is_playlist: bool,
    config: Config,

    show_modal: bool,
    active_tab: usize,
    modal_body: String,
    modal_title: String,

    sender: Option<UnboundedSender<String>>,
    command: command::Command,
    progress: f32,
}

impl YtGUI {
    pub fn command_update(&mut self, message: command::Message) {
        match message {
            command::Message::Run(link) => {
                self.config
                    .update_config_file()
                    .expect("update config file");

                let mut args: Vec<&str> = Vec::new();

                if link.is_empty() {
                    self.show_modal = true;
                    self.modal_body = String::from("No Download link was provided!");
                    self.modal_title = String::from("Error");
                    return;
                }

                args.push(&link);

                match self.active_tab {
                    0 => {
                        // Video tab

                        args.push("-S");

                        args.push(self.config.options.video_resolution.options());

                        // after downloading a video with a specific format
                        // yt-dlp sometimes downloads the audio and video seprately
                        // then merge them in a different format
                        // this enforces the chosen format by the user
                        args.push("--remux-video");

                        args.push(self.config.options.video_format.options());

                        tracing::info!("{args:#?}");
                    }
                    1 => {
                        // Audio tab

                        // Extract audio from Youtube video
                        args.push("-x");

                        args.push("--audio-format");
                        args.push(self.config.options.audio_format.options());

                        args.push("--audio-quality");
                        args.push(self.config.options.audio_quality.options());
                    }
                    _ => {}
                }

                let playlist_options =
                    playlist_options(self.is_playlist, self.config.download_folder.clone());

                args.append(&mut playlist_options.iter().map(|s| &**s).collect());

                self.modal_title = String::from("Initializing");
                self.command.start(
                    args,
                    &mut self.show_modal,
                    &mut self.modal_body,
                    self.config.bin_dir.clone(),
                    self.sender.clone(),
                );
            }
            command::Message::Stop => {
                match self.command.kill() {
                    Ok(_) => {
                        tracing::debug!("killed child process")
                    }
                    Err(e) => {
                        tracing::error!("failed to kill child process {e}")
                    }
                };
                self.show_modal = false;
                self.progress = 0.;
                self.modal_body.clear();
            }
            command::Message::AlreadyExists => {
                match self.command.kill() {
                    Ok(_) => {
                        tracing::debug!("killed child process")
                    }
                    Err(e) => {
                        tracing::error!("failed to kill child process {e}")
                    }
                };
                self.progress = 0.;
                self.modal_body = String::from("Already exists");
                self.modal_title = String::from("Error");
            }
            command::Message::PlaylistNotChecked => {
                match self.command.kill() {
                    Ok(_) => {
                        tracing::debug!("killed child process")
                    }
                    Err(e) => {
                        tracing::error!("failed to kill child process {e}")
                    }
                };
                self.progress = 0.;
                self.modal_body = String::from("Playlist checkbox not checked!");
                self.modal_title = String::from("Error");
            }
            command::Message::Finished => {
                match self.command.kill() {
                    Ok(_) => {
                        tracing::debug!("killed child process")
                    }
                    Err(e) => {
                        tracing::error!("failed to kill child process {e}")
                    }
                };
                self.modal_title = String::from("Done");
                self.modal_body = String::from("Finished!");
                self.log_download();
            }
            command::Message::Error(e) => {
                self.modal_title = String::from("Error");
                self.progress = 0.;

                if e.contains("Private video. Sign in if you've been granted access to this video")
                {
                    self.modal_body = String::from("Private video, skipping...");
                } else if e.contains("Video unavailable. This video contains content") {
                    self.modal_body = String::from("Video unavailable, skipping...");
                } else {
                    self.modal_body = String::from("Something went wrong, logging...");
                }

                tracing::error!("failed to download: {e}");
            }
        }
    }
    fn log_download(&self) {
        let downloads_log_path = dirs::cache_dir()
            .expect("config directory")
            .join("ytdlp-gui/downloads.log");

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(downloads_log_path)
            .expect("downloads logs file");

        // [<date-time>]::<URL>::<options>::<download-path>
        if let Err(e) = writeln!(
            file,
            "{}::{}::{}::{}",
            Local::now(),
            self.download_link,
            if self.active_tab == 1 {
                format!(
                    "{:?}:{:?}",
                    self.config.options.video_resolution, self.config.options.video_format
                )
            } else {
                format!(
                    "{:?}:{:?}",
                    self.config.options.audio_quality, self.config.options.audio_format
                )
            },
            self.config
                .download_folder
                .clone()
                .unwrap_or_else(|| "~/Videos".into())
                .to_string_lossy()
        ) {
            tracing::error!("failed to log download: {e}");
        }
    }
}

impl Application for YtGUI {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = Config;
    type Theme = theme::Theme;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Message>) {
        tracing::info!("config loaded: {flags:#?}");

        (
            Self {
                download_link: String::default(),
                is_playlist: bool::default(),
                config: flags,

                show_modal: false,
                active_tab: 0,
                modal_body: String::default(),
                modal_title: String::from("Downloading"),

                sender: None,
                command: command::Command::default(),
                progress: 0.,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        "Youtube Downloader".to_string()
    }

    fn update(&mut self, event: Message) -> iced::Command<Message> {
        match event {
            Message::Command(message) => {
                self.command_update(message);
            }
            Message::InputChanged(input) => {
                self.download_link = input;
            }
            Message::SelectedResolution(resolution) => {
                self.config.options.video_resolution = resolution;
            }
            Message::TogglePlaylist(is_playlist) => {
                self.is_playlist = is_playlist;
            }
            Message::SelectedVideoFormat(format) => {
                self.config.options.video_format = format;
            }
            Message::SelectFolder => {
                if let Ok(Some(path)) = FileDialog::new()
                    .set_location(
                        &self
                            .config
                            .download_folder
                            .clone()
                            .unwrap_or_else(|| "~/Videos".into()),
                    )
                    .show_open_single_dir()
                {
                    self.config.download_folder = Some(path);
                }
            }
            Message::SelectFolderTextInput(folder_string) => {
                let path = PathBuf::from(folder_string);

                self.config.download_folder = Some(path);
            }
            Message::SelectTab(tab_number) => {
                self.active_tab = tab_number;
            }
            Message::SelectedAudioFormat(format) => {
                self.config.options.audio_format = format;
            }
            Message::SelectedAudioQuality(quality) => {
                self.config.options.audio_quality = quality;
            }
            Message::ProgressEvent(progress) => {
                match parse_progress(progress.clone()) {
                    Some(Progress::Downloading {
                        video_title: _,
                        eta,
                        downloaded_bytes,
                        total_bytes,
                        elapsed: _,
                        speed,
                        playlist_count,
                        playlist_index,
                    }) => {
                        self.progress = (downloaded_bytes / total_bytes) * 100.;
                        if let Some((playlist_count, playlist_index)) =
                            playlist_count.zip(playlist_index)
                        {
                            self.modal_title =
                                format!("Downloading {} - {}", playlist_index, playlist_count);
                        } else {
                            self.modal_title = String::from("Downloading");
                        }

                        let eta = chrono::Duration::seconds(eta.into());

                        let downloaded_megabytes = downloaded_bytes / 1024_f32.powi(2);
                        let total_downloaded = if downloaded_megabytes > 1024. {
                            format!("{:.2}GB", downloaded_megabytes / 1024.)
                        } else {
                            format!("{:.2}MB", downloaded_megabytes)
                        };

                        self.modal_body = format!(
                            "{total_downloaded} | {speed:.2}MB/s | ETA {eta_mins}:{eta_secs}",
                            speed = speed / 1024_f32.powi(2),
                            // percent = if self.progress.is_infinite() {
                            //     100.
                            // } else {
                            //     self.progress
                            // },
                            eta_mins = eta.num_minutes(),
                            eta_secs = eta.num_seconds() - (eta.num_minutes() * 60),
                        );
                    }
                    Some(Progress::PostProcessing { status: _ }) => {
                        self.modal_title = String::from("Processing");
                        self.modal_body = String::from("Processing...");
                    }
                    Some(Progress::EndOfPlaylist) => {
                        match self.command.kill() {
                            Ok(_) => {
                                tracing::debug!("killed child process")
                            }
                            Err(e) => {
                                tracing::error!("failed to kill child process {e}")
                            }
                        };
                        self.modal_title = String::from("Done");
                        self.modal_body = String::from("Finished playlist!");
                        self.log_download();
                    }
                    Some(Progress::EndOfVideo) => {
                        if !self.is_playlist {
                            match self.command.kill() {
                                Ok(_) => {
                                    tracing::debug!("killed child process")
                                }
                                Err(e) => {
                                    tracing::error!("failed to kill child process {e}")
                                }
                            };
                            self.modal_title = String::from("Done");
                            self.modal_body = String::from("Finished!");
                            self.log_download();
                        }
                    }
                    Some(_) => {}
                    None => {
                        if progress.contains("Encountered a video that did not match filter") {
                            self.modal_body = String::from(
                                "Playlist box needs to be checked to download a playlist",
                            );
                        }
                    }
                }

                return iced::Command::none();
            }
            Message::Ready(sender) => {
                self.sender = Some(sender);
            }
            Message::IcedEvent(event) => {
                if let iced_native::Event::Window(iced_native::window::Event::CloseRequested) =
                    event
                {
                    if self.command.kill().is_ok() {
                        tracing::debug!("killed child process");
                    }
                    return iced::Command::single(iced_native::command::Action::Window(
                        iced_native::window::Action::Close,
                    ));
                }
            }
            Message::None => {}
        }

        iced::Command::none()
    }

    fn view(&self) -> widgets::Element<Message> {
        let content: widgets::Element<Message> = column![
            row![
                text("Enter URL: "),
                text_input("Download link", &self.download_link)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::Command(command::Message::Run(
                        self.download_link.clone(),
                    )))
                    .size(FONT_SIZE)
                    .width(Length::Fill),
                checkbox("Playlist", self.is_playlist, Message::TogglePlaylist)
            ]
            .spacing(7)
            .align_items(iced::Alignment::Center),
            Tabs::new(self.active_tab, Message::SelectTab)
                .push(
                    iced_aw::TabLabel::Text("Video".to_string()),
                    column![
                        Options::video_resolutions(self.config.options.video_resolution)
                            .width(Length::Fill),
                        Options::video_formats(self.config.options.video_format),
                    ],
                )
                .push(
                    iced_aw::TabLabel::Text("Audio".to_string()),
                    column![
                        Options::audio_qualities(self.config.options.audio_quality),
                        Options::audio_formats(self.config.options.audio_format),
                    ],
                )
                .height(Length::Shrink)
                .width(Length::FillPortion(1))
                .tab_bar_width(Length::FillPortion(1)),
            row![
                text_input(
                    "",
                    &self
                        .config
                        .download_folder
                        .clone()
                        .unwrap_or_else(|| "~/Videos".into())
                        .to_string_lossy()
                )
                .on_input(Message::SelectFolderTextInput),
                button("Browse").on_press(Message::SelectFolder),
            ]
            .spacing(SPACING)
            .align_items(iced::Alignment::Center),
            row![
                button("Download").on_press(Message::Command(command::Message::Run(
                    self.download_link.clone(),
                ))),
            ]
        ]
        .width(Length::Fill)
        .align_items(iced::Alignment::Center)
        .spacing(20)
        .padding(20)
        .into();

        let content = Modal::new(self.show_modal, content, || {
            Card::new(
                text(&*self.modal_title)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center),
                column![
                    text(self.modal_body.clone())
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                        .height(Length::Fill),
                    row![progress_bar(0.0..=100., self.progress)]
                ]
                .align_items(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .max_height(70.)
            .max_width(300.)
            .on_close(Message::Command(command::Message::Stop))
            .into()
        });

        // let content = content.explain(Color::BLACK);

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let iced_events = subscription::events().map(Message::IcedEvent);
        Subscription::batch(vec![bind(), iced_events])
    }
}

pub fn logging() {
    if let Err(_e) = std::env::var("YTG_LOG") {
        tracing::info!(
            "no log level specified, defaulting to debug level for ytdlp_gui crate only"
        );
        std::env::set_var("YTG_LOG", "none,ytdlp_gui=debug");
    }

    let logs_dir = dirs::cache_dir()
        .expect("cache dir should exist")
        .join("ytdlp-gui/logs");

    // Log all `tracing` events to files prefixed with `debug`. Since these
    // files will be written to very frequently, roll the log file every minute.
    let debug_file = rolling::minutely(&logs_dir, "debug");
    // Log warnings and errors to a separate file. Since we expect these events
    // to occur less frequently, roll that file on a daily basis instead.
    let warn_file = rolling::daily(&logs_dir, "warnings");

    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_env_var("YTG_LOG")
                .with_default_directive(LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(debug_file.with_max_level(Level::DEBUG))
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(warn_file.with_max_level(tracing::Level::WARN))
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(std::io::stdout.with_max_level(Level::DEBUG)),
        )
        .init();
}
