use std::path::PathBuf;

use iced::widget::{pick_list, row, text};
use serde::{Deserialize, Serialize};

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

impl core::fmt::Display for VideoResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoResolution::FourK => write!(f, "4K"),
            VideoResolution::TwoK => write!(f, "1440p"),
            VideoResolution::FullHD => write!(f, "1080p"),
            VideoResolution::Hd => write!(f, "720p"),
            VideoResolution::Sd => write!(f, "480p"),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoFormat {
    #[default]
    Mp4,
    Mkv,
    Webm,
    // Flv,
}

impl core::fmt::Display for VideoFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoFormat::Mp4 => write!(f, "MP4"),
            VideoFormat::Mkv => write!(f, "MKV"),
            VideoFormat::Webm => write!(f, "WEBM"),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioQuality {
    Best,
    #[default]
    Good,
    Medium,
    Low,
}

impl core::fmt::Display for AudioQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioQuality::Best => write!(f, "Best"),
            AudioQuality::Good => write!(f, "Good"),
            AudioQuality::Medium => write!(f, "Medium"),
            AudioQuality::Low => write!(f, "Low"),
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    #[default]
    Mp3,
    Wav,
    Vorbis,
    M4a,
    Opus,
}

impl core::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Vorbis => write!(f, "VORBIS"),
            AudioFormat::M4a => write!(f, "M4A"),
            AudioFormat::Opus => write!(f, "OPUS"),
        }
    }
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
    pub fn options(&self) -> String {
        match self {
            AudioQuality::Best => String::from("0"),
            AudioQuality::Good => String::from("2"),
            AudioQuality::Medium => String::from("4"),
            AudioQuality::Low => String::from("6"),
        }
    }
}

impl Options {
    pub fn video_resolutions(resolution: VideoResolution) -> widgets::Row<'static, Message> {
        row![
            text("Resolution: ").size(FONT_SIZE),
            pick_list(
                vec![
                    VideoResolution::FourK,
                    VideoResolution::TwoK,
                    VideoResolution::FullHD,
                    VideoResolution::Hd,
                    VideoResolution::Sd
                ],
                Some(resolution),
                Message::SelectedResolution
            )
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
            pick_list(
                vec![VideoFormat::Mp4, VideoFormat::Mkv, VideoFormat::Webm],
                Some(format),
                Message::SelectedVideoFormat
            )
            .text_size(FONT_SIZE),
        ]
        .width(iced::Length::Fill)
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }

    pub fn audio_formats(format: AudioFormat) -> widgets::Row<'static, Message> {
        row![
            text("Preferred Format: ").size(FONT_SIZE),
            pick_list(
                vec![
                    AudioFormat::Mp3,
                    AudioFormat::Wav,
                    AudioFormat::Vorbis,
                    AudioFormat::M4a,
                    AudioFormat::Opus,
                ],
                Some(format),
                Message::SelectedAudioFormat
            )
            .text_size(FONT_SIZE),
        ]
        .width(iced::Length::Fill)
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }

    pub fn audio_qualities(quality: AudioQuality) -> widgets::Row<'static, Message> {
        row![
            text("Quality: ").size(FONT_SIZE),
            pick_list(
                vec![
                    AudioQuality::Best,
                    AudioQuality::Good,
                    AudioQuality::Medium,
                    AudioQuality::Low,
                ],
                Some(quality),
                Message::SelectedAudioQuality
            )
            .text_size(FONT_SIZE)
        ]
        .width(iced::Length::Fill)
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
                .unwrap_or_else(|| "~/Videos".into())
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
                .unwrap_or_else(|| "~/Videos".into())
                .to_str()
                .expect("download folder as str")
                .to_string(),
        );
        args.push(String::from("-o"));
        args.push(String::from("%(title)s.%(ext)s"));
    }

    args
}
