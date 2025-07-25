use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

use app::{DownloadType, Tab};
use error::DownloadError;
#[cfg(feature = "explain")]
use iced::Color;

use chrono::Local;
use iced::futures::channel::mpsc::UnboundedSender;
use iced::{Event, Point};

use rfd::AsyncFileDialog;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};

mod app;
pub mod command;
mod error;
pub mod i18n;
pub mod media_options;
pub mod progress;
mod sponsorblock;
pub mod theme;

use sponsorblock::SponsorBlockOption;
use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::media_options::Options;
use crate::media_options::{AudioFormat, AudioQuality, VideoFormat, VideoResolution};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    TogglePlaylist(bool),
    SelectedSponsorBlockOption(SponsorBlockOption),
    SelectedVideoFormat(VideoFormat),
    SelectedResolution(VideoResolution),
    SelectedAudioFormat(AudioFormat),
    SelectedAudioQuality(AudioQuality),
    SelectDownloadFolder,
    SelectedDownloadFolder(Option<PathBuf>),
    DownloadFolderTextInput(String),
    SelectDownloadFolderTextInput,
    SelectTab(Tab),
    ProgressEvent(String),
    StartDownload(String),
    StopDownload,
    IcedEvent(Event),
    ToggleSaveWindowPosition(bool),
    SelectYtDlpBinPath,
    SelectedYtDlpBinPath(Option<PathBuf>),
    SelectYtDlpBitPathTextInput(String),
    SelectCookiesFile,
    SelectedCookiesFile(Option<PathBuf>),
    SelectCookiesFileTextInput(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub struct Flags {
    pub url: Option<String>,
    pub config: Config,
}

fn download_folder_default() -> PathBuf {
    shellexpand::tilde("~/Videos").to_string().into()
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_deref();
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(deserialize_with = "empty_string_as_none")]
    bin_path: Option<PathBuf>,
    #[serde(default = "download_folder_default")]
    download_folder: PathBuf,
    #[serde(deserialize_with = "empty_string_as_none")]
    cookies_file: Option<PathBuf>,
    pub save_window_position: bool,
    pub window_position: Option<WindowPosition>,
    pub window_size: Option<WindowSize>,
    options: Options,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bin_path: Default::default(),
            download_folder: download_folder_default(),
            cookies_file: Default::default(),
            save_window_position: Default::default(),
            window_position: Default::default(),
            window_size: Default::default(),
            options: Default::default(),
        }
    }
}

impl Config {
    fn update_config_file(&mut self) -> io::Result<()> {
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
    sponsorblock: Option<SponsorBlockOption>,
    config: Config,

    active_tab: Tab,
    download_type: DownloadType,
    playlist_progress: Option<String>,
    download_message: Option<Result<String, DownloadError>>,
    is_file_dialog_open: bool,
    download_text_input_id: iced::widget::text_input::Id,

    sender: UnboundedSender<Message>,
    command: command::Command,
    progress: Option<f32>,
    window_height: f32,
    window_width: f32,
    window_pos: Point,
}

impl YtGUI {
    pub fn new(
        flags: Flags,
        progress_sender: iced::futures::channel::mpsc::UnboundedSender<Message>,
    ) -> Self {
        tracing::info!("config loaded: {flags:#?}");

        Self {
            download_link: flags.url.clone().unwrap_or_default(),
            is_playlist: Default::default(),
            sponsorblock: Default::default(),
            config: flags.config,

            active_tab: Tab::Video,
            download_type: DownloadType::Video,
            playlist_progress: None,
            download_message: Default::default(),
            download_text_input_id: iced::widget::text_input::Id::unique(),

            sender: progress_sender,
            command: command::Command::default(),
            progress: None,
            window_height: 0.,
            window_width: 0.,
            is_file_dialog_open: false,
            window_pos: Point::default(),
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
            match self.download_type {
                DownloadType::Video => format!(
                    "{:?}:{:?}",
                    self.config.options.video_resolution, self.config.options.video_format
                ),
                DownloadType::Audio => format!(
                    "{:?}:{:?}",
                    self.config.options.audio_quality, self.config.options.audio_format
                ),
            },
            self.config.download_folder.to_string_lossy()
        ) {
            tracing::error!("failed to log download: {e}");
        }
    }
}

async fn choose_folder(starting_dir: impl AsRef<Path>) -> Option<PathBuf> {
    AsyncFileDialog::new()
        .set_directory(starting_dir)
        .pick_folder()
        .await
        .map(|f| f.path().to_path_buf())
}

async fn choose_file(starting_dir: impl AsRef<Path>) -> Option<PathBuf> {
    AsyncFileDialog::new()
        .set_directory(starting_dir)
        .pick_file()
        .await
        .map(|f| f.path().to_path_buf())
}

pub fn logging() {
    if let Err(_e) = std::env::var("YTG_LOG") {
        tracing::info!(
            "no log level specified, defaulting to debug level for ytdlp_gui crate only"
        );
        unsafe { std::env::set_var("YTG_LOG", "none,ytdlp_gui=debug") };
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

#[macro_export]
macro_rules! git_hash {
    () => {
        match option_env!("GIT_HASH") {
            Some(hash) => hash.to_string(),
            None => {
                let output = std::process::Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .output()
                    .unwrap();
                String::from_utf8(output.stdout).unwrap()
            }
        }
    };
}
