use std::path::PathBuf;

use iced::widget::{radio, row, text};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{widgets, Message, FONT_SIZE, SPACING};

#[derive(Deserialize, Serialize, Debug, Default, Copy, Clone)]
pub struct Options {
    pub video_resolution: VideoResolution,
    pub video_format: VideoFormat,
    pub audio_quality: AudioQuality,
    pub audio_format: AudioFormat,
}

#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoResolution {
    FourK,
    TwoK,
    #[default]
    FullHD,
    Hd,
    Sd,
}

#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoFormat {
    #[default]
    Mp4,
    Mkv,
    Webm,
    // Flv,
}

#[derive(Deserialize, Serialize, Display, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioQuality {
    Best,
    #[default]
    Good,
    Medium,
    Low,
}

#[derive(Deserialize, Serialize, Display, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    #[default]
    Mp3,
    Wav,
    Vorbis,
    M4a,
    Opus,
}

impl VideoResolution {
    pub fn options(&self) -> &str {
        match self {
            VideoResolution::FourK => "res:2160",
            VideoResolution::TwoK => "res:1440",
            VideoResolution::FullHD => "res:1080",
            VideoResolution::Hd => "res:720",
            VideoResolution::Sd => "res:480",
        }
    }
}

impl VideoFormat {
    pub fn options(&self) -> &str {
        match self {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Mkv => "mkv",
            VideoFormat::Webm => "webm",
        }
    }
}

impl AudioFormat {
    const ALL: [AudioFormat; 5] = [
        AudioFormat::Mp3,
        AudioFormat::Wav,
        AudioFormat::Vorbis,
        AudioFormat::Opus,
        AudioFormat::M4a,
    ];

    pub fn options(&self) -> String {
        match self {
            AudioFormat::Mp3 => String::from("mp3"),
            AudioFormat::Wav => String::from("wav"),
            AudioFormat::Vorbis => String::from("vorbis"),
            AudioFormat::Opus => String::from("opus"),
            AudioFormat::M4a => String::from("m4a"),
        }
    }
}

impl AudioQuality {
    const ALL: [AudioQuality; 4] = [
        AudioQuality::Best,
        AudioQuality::Good,
        AudioQuality::Medium,
        AudioQuality::Low,
    ];

    pub fn options(&self) -> String {
        match self {
            AudioQuality::Best => String::from("0"),
            AudioQuality::Good => String::from("2"),
            AudioQuality::Medium => String::from("4"),
            AudioQuality::Low => String::from("6"),
        }
    }
}

const RADIO_DOT_SIZE: u16 = 15;

impl Options {
    pub fn video_resolutions(resolution: VideoResolution) -> widgets::Row<'static, Message> {
        row![
            text("Resolution: ").size(FONT_SIZE),
            radio(
                "4K",
                VideoResolution::FourK,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            radio(
                "1440p",
                VideoResolution::TwoK,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            radio(
                "1080p",
                VideoResolution::FullHD,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            radio(
                "720p",
                VideoResolution::Hd,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            radio(
                "480p",
                VideoResolution::Sd,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
        ]
        .spacing(SPACING)
        .width(iced::Length::Fill)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }

    pub fn video_formats(format: VideoFormat) -> widgets::Row<'static, Message> {
        row![
            text("Preferred Format: ").size(FONT_SIZE),
            radio(
                "MP4",
                VideoFormat::Mp4,
                Some(format),
                Message::SelectedVideoFormat,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            radio(
                "WEBM",
                VideoFormat::Webm,
                Some(format),
                Message::SelectedVideoFormat,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            radio(
                "MKV",
                VideoFormat::Mkv,
                Some(format),
                Message::SelectedVideoFormat,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
        ]
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }

    pub fn audio_formats(format: AudioFormat) -> widgets::Row<'static, Message> {
        row![
            text("Preferred Format: ").size(FONT_SIZE),
            AudioFormat::ALL
                .iter()
                .cloned()
                .fold(row![], |row, audio_format| {
                    row.push(
                        radio(
                            audio_format.to_string().to_ascii_uppercase(),
                            audio_format,
                            Some(format),
                            Message::SelectedAudioFormat,
                        )
                        .size(RADIO_DOT_SIZE)
                        .text_size(FONT_SIZE),
                    )
                    .spacing(SPACING)
                }),
        ]
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }

    pub fn audio_qualities(quality: AudioQuality) -> widgets::Row<'static, Message> {
        row![
            text("Quality: ").size(FONT_SIZE),
            AudioQuality::ALL
                .iter()
                .cloned()
                .fold(row![], |row, audio_quality| {
                    row.push(
                        radio(
                            audio_quality.to_string(),
                            audio_quality,
                            Some(quality),
                            Message::SelectedAudioQuality,
                        )
                        .size(RADIO_DOT_SIZE)
                        .text_size(FONT_SIZE),
                    )
                    .spacing(SPACING)
                }),
        ]
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }
}

pub fn playlist_options(is_playlist: bool, download_folder: Option<PathBuf>) -> Vec<String> {
    let mut args = Vec::new();
    if is_playlist {
        args.push(String::from("--yes-playlist"));
        args.push(String::from("-P"));
        args.push(
            download_folder
                .clone()
                .unwrap_or("~/Videos".into())
                .to_str()
                .expect("directory as str")
                .to_string(),
        );
        args.push(String::from("-o %(playlist)s/%(title)s.%(ext)s"));
    } else {
        args.push(String::from("--break-on-reject"));
        args.push(String::from("--match-filter"));
        args.push(String::from("!playlist"));
        args.push(String::from("--no-playlist"));
        args.push(String::from("-P"));
        args.push(
            download_folder
                .clone()
                .unwrap_or("~/Videos".into())
                .to_str()
                .expect("download folder as str")
                .to_string(),
        );
        args.push(String::from("-o"));
        args.push(String::from("%(title)s.%(ext)s"));
    }

    args
}
