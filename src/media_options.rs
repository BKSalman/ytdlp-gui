use std::path::PathBuf;

use iced::widget::{pick_list, row, text};
use serde::{Deserialize, Serialize};

use crate::{
    app::{FONT_SIZE, SPACING},
    fl,
    theme::{pick_list_menu_style, pick_list_style},
    Message,
};

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
            AudioQuality::Best => f.write_str(&fl!("quality_best")),
            AudioQuality::Good => f.write_str(&fl!("quality_good")),
            AudioQuality::Medium => f.write_str(&fl!("quality_medium")),
            AudioQuality::Low => f.write_str(&fl!("quality_low")),
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
    pub fn options(&self) -> &str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Wav => "wav",
            AudioFormat::Vorbis => "vorbis",
            AudioFormat::Opus => "opus",
            AudioFormat::M4a => "m4a",
        }
    }
}

impl AudioQuality {
    pub fn options(&self) -> &str {
        match self {
            AudioQuality::Best => "0",
            AudioQuality::Good => "2",
            AudioQuality::Medium => "4",
            AudioQuality::Low => "6",
        }
    }
}

impl Options {
    pub fn video_resolutions(resolution: VideoResolution) -> iced::widget::Row<'static, Message> {
        row![
            text(format!("{}: ", fl!("resolution"))).size(FONT_SIZE),
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
            .text_size(FONT_SIZE)
            .menu_style(pick_list_menu_style)
            .style(pick_list_style),
        ]
        .spacing(SPACING)
        .width(iced::Length::Fill)
        .align_y(iced::Alignment::Center)
        .padding(12)
    }

    pub fn video_formats(format: VideoFormat) -> iced::widget::Row<'static, Message> {
        row![
            text(format!("{}: ", fl!("preferred_format"))).size(FONT_SIZE),
            pick_list(
                vec![VideoFormat::Mp4, VideoFormat::Mkv, VideoFormat::Webm],
                Some(format),
                Message::SelectedVideoFormat
            )
            .text_size(FONT_SIZE)
            .menu_style(pick_list_menu_style)
            .style(pick_list_style),
        ]
        .width(iced::Length::Fill)
        .spacing(SPACING)
        .align_y(iced::Alignment::Center)
        .padding(12)
    }

    pub fn audio_formats(format: AudioFormat) -> iced::widget::Row<'static, Message> {
        row![
            text(format!("{}: ", fl!("preferred_format"))).size(FONT_SIZE),
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
            .text_size(FONT_SIZE)
            .style(pick_list_style)
            .menu_style(pick_list_menu_style)
        ]
        .width(iced::Length::Fill)
        .spacing(SPACING)
        .align_y(iced::Alignment::Center)
        .padding(12)
    }

    pub fn audio_qualities(quality: AudioQuality) -> iced::widget::Row<'static, Message> {
        row![
            text(format!("{}: ", fl!("quality"))).size(FONT_SIZE),
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
            .style(pick_list_style)
            .menu_style(pick_list_menu_style)
        ]
        .width(iced::Length::Fill)
        .spacing(SPACING)
        .align_y(iced::Alignment::Center)
        .padding(12)
    }
}

pub fn playlist_options(is_playlist: bool, download_folder: PathBuf) -> Vec<String> {
    let download_dir = download_folder.to_string_lossy().to_string();

    if is_playlist {
        vec![
            String::from("--yes-playlist"),
            String::from("-P"),
            download_dir,
            String::from("-o"),
            String::from("%(playlist)s/%(title)s.%(ext)s"),
        ]
    } else {
        vec![
            String::from("--break-on-reject"),
            String::from("--match-filter"),
            String::from("!playlist"),
            String::from("--no-playlist"),
            String::from("-P"),
            download_dir,
            String::from("-o"),
            String::from("%(title)s.%(ext)s"),
        ]
    }
}
