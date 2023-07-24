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

use log::{error, info, LevelFilter};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::Appender;
use log4rs::config::Root;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use native_dialog::FileDialog;
use serde::{Deserialize, Serialize};
// use theme::widget::Element;

pub mod command;
pub mod media_options;
pub mod progress;
pub mod theme;
pub mod widgets;

use widgets::{Modal, Tabs};

use crate::media_options::{playlist_options, Options};
use crate::media_options::{AudioFormat, AudioQuality, VideoFormat, VideoResolution};
use crate::progress::bind;

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
        info!("Updated config file to {}", current_config);
        Ok(())
    }
}

pub struct YtGUI {
    download_link: String,
    is_playlist: bool,
    config: Config,

    show_modal: bool,
    active_tab: usize,
    ui_message: String,

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

                let mut args = Vec::new();

                if link.is_empty() {
                    self.show_modal = true;
                    self.ui_message = String::from("No Download link was provided!");
                    return;
                }

                args.push(link);

                match self.active_tab {
                    0 => {
                        // Video tab

                        args.push(String::from("-S"));

                        args.push(self.config.options.video_resolution.options().to_string());

                        // after downloading a video with a specific format
                        // yt-dlp sometimes downloads the audio and video seprately
                        // then merge them in a different format
                        // this enforces the chosen format by the user
                        args.push(String::from("--remux-video"));

                        args.push(self.config.options.video_format.options().to_string());

                        info!("{args:#?}");
                    }
                    1 => {
                        // Audio tab

                        // Extract audio from Youtube video
                        args.push(String::from("-x"));

                        args.push(String::from("--audio-format"));
                        args.push(self.config.options.audio_format.options());

                        args.push(String::from("--audio-quality"));
                        args.push(self.config.options.audio_quality.options());
                    }
                    _ => {}
                }

                args.append(&mut playlist_options(
                    self.is_playlist,
                    self.config.download_folder.clone(),
                ));

                self.command.start(
                    args,
                    &mut self.show_modal,
                    &mut self.ui_message,
                    self.config.bin_dir.clone(),
                    self.sender.clone(),
                );
            }
            command::Message::Stop => {
                match self.command.kill() {
                    Ok(_) => {
                        info!("killed child process")
                    }
                    Err(e) => {
                        error!("{e}")
                    }
                };
                self.show_modal = false;
                self.progress = 0.;
                self.ui_message.clear();
            }
            command::Message::Finished => {
                match self.command.kill() {
                    Ok(_) => {
                        info!("killed child process")
                    }
                    Err(e) => {
                        error!("{e}")
                    }
                };
                self.progress = 0.;
                if self.ui_message.contains("Already") {
                    return;
                }
                self.ui_message = String::from("Finished!");
                self.log_download();
            }
        }
    }
    fn log_download(&self) {
        let downloads_log_path = dirs::config_dir()
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
                .to_str()
                .expect("path to string"),
        ) {
            error!("{e}");
        }
    }
}

impl Application for YtGUI {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = Config;
    type Theme = theme::Theme;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Message>) {
        info!("{flags:#?}");

        (
            Self {
                download_link: String::default(),
                is_playlist: bool::default(),
                config: flags,

                show_modal: false,
                active_tab: 0,
                ui_message: String::default(),

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
                        self.config
                            .download_folder
                            .clone()
                            .unwrap_or_else(|| "~/Videos".into())
                            .to_str()
                            .expect("download folder as str"),
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
                if self.progress == 100. {
                    self.ui_message = String::from("Processing...");
                    return iced::Command::none();
                }

                if progress.contains('%') {
                    let words = progress
                        .split(' ')
                        .map(String::from)
                        .filter(|str| str.chars().filter(|char| char.is_numeric()).count() != 0)
                        .collect::<Vec<String>>();

                    if let Ok(percentage) = words[0].trim_end_matches('%').parse::<f32>() {
                        self.progress = percentage;
                    }

                    self.ui_message = words[1..].join(" | ");

                    return iced::Command::none();
                } else if progress.contains("[ExtractAudio]") {
                    self.ui_message = String::from("Extracting audio");
                    return iced::Command::none();
                } else if progress.contains("has already been downloaded") {
                    self.ui_message = String::from("Already downloaded");
                    return iced::Command::none();
                } else if progress.contains("Encountered a video that did not match filter") {
                    self.ui_message =
                        String::from("Playlist box needs to be checked to download a playlist");
                    return iced::Command::none();
                }

                info!("{progress}");
            }
            Message::Ready(sender) => {
                self.sender = Some(sender);
            }
            Message::IcedEvent(event) => {
                if let iced_native::Event::Window(iced_native::window::Event::CloseRequested) =
                    event
                {
                    if self.command.kill().is_ok() {
                        info!("killed child process");
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
                    self.config
                        .download_folder
                        .clone()
                        .unwrap_or_else(|| "~/Videos".into())
                        .to_str()
                        .unwrap()
                )
                .on_input(Message::SelectFolderTextInput),
                button("Browse").on_press(Message::SelectFolder),
            ]
            .spacing(SPACING)
            .align_items(iced::Alignment::Center),
            row![
                button(text("Download")).on_press(Message::Command(command::Message::Run(
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
                text("Downloading")
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center),
                column![
                    text(self.ui_message.clone())
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

pub fn setup_logger() {
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    let temporary_dir = std::env::temp_dir();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(temporary_dir.join("ytdlp-gui.log"))
        .expect("file appender");

    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .expect("logger config");

    log4rs::init_config(config).unwrap();
}
