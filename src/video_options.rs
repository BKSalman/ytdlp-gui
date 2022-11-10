use iced::widget::{row, radio, text, Row};
use strum::Display;

use crate::{Message, FONT_SIZE, SPACING};

#[derive(Default, Copy, Clone)]
pub struct Options {
    pub video_resolution: VideoResolution,
    pub video_format: VideoFormat,
    pub audio_quality: AudioQuality,
    pub audio_format: AudioFormat,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoResolution {
    FourK,
    TwoK,
    #[default]
    FullHD,
    Hd,
    Sd,
}

#[derive(Display, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoFormat {
    #[default]
    Mp4,
    ThreeGP,
    Webm,
    // Flv,
}

#[derive(Display, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioQuality {
    Best,
    #[default]
    Good,
    Medium,
    Low,
}

#[derive(Display, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    #[default]
    Mp3,
    Wav,
    Vorbis,
    M4a,
    Opus,
}

impl AudioFormat {
    const ALL: [AudioFormat; 5] = [
        AudioFormat::Mp3,
        AudioFormat::Wav,
        AudioFormat::Vorbis,
        AudioFormat::Opus,
        AudioFormat::M4a,
    ];
}

impl AudioQuality {
    const ALL: [AudioQuality; 4] = [
        AudioQuality::Best,
        AudioQuality::Good,
        AudioQuality::Medium,
        AudioQuality::Low,
    ];
}

const RADIO_DOT_SIZE: u16 = 15;

impl Options {
    pub fn video_resolutions(resolution: VideoResolution) -> Row<'static, Message> {
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
            // .style(theme),
            radio(
                "1440p",
                VideoResolution::TwoK,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            // .style(theme),
            radio(
                "1080p",
                VideoResolution::FullHD,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            // .style(theme),
            radio(
                "720p",
                VideoResolution::Hd,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            // .style(theme),
            radio(
                "480p",
                VideoResolution::Sd,
                Some(resolution),
                Message::SelectedResolution,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            // .style(theme),
        ]
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }

    pub fn video_formats(format: VideoFormat) -> Row<'static, Message> {
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
            // .style(theme),
            radio(
                "WEBM",
                VideoFormat::Webm,
                Some(format),
                Message::SelectedVideoFormat,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE),
            // .style(theme),
            radio(
                "3GP",
                VideoFormat::ThreeGP,
                Some(format),
                Message::SelectedVideoFormat,
            )
            .size(RADIO_DOT_SIZE)
            .text_size(FONT_SIZE) // .style(theme),
        ]
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }
    pub fn audio_formats(format: AudioFormat) -> Row<'static, Message> {
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
                        // .style(theme),
                    )
                    .spacing(SPACING)
                }),
        ]
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }

    pub fn audio_qualities(quality: AudioQuality) -> Row<'static, Message> {
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
                        // .style(theme),
                    )
                    .spacing(SPACING)
                }),
        ]
        .spacing(SPACING)
        .align_items(iced::Alignment::Center)
        .padding(12)
    }
}
