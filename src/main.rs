use std::{path::PathBuf, process::Command};

use iced::{
    button, text_input::State, window, Button, Checkbox, Color, Column, Container, Element, Length,
    Radio, Row, Sandbox, Settings, Text, TextInput,
};

use iced_aw::Tabs;

use native_dialog::FileDialog;

use strum::Display;

mod theme;
use theme::Theme;

const FONT_SIZE: u16 = 18;

const SPACING: u16 = 10;

#[derive(Debug, Clone)]
pub enum Message {
    Download(String),
    InputChanged(String),
    TogglePlaylist(bool),
    SelectedVideoFormat(VideoFormat),
    SelectedResolution(Resolution),
    SelectedAudioFormat(AudioFormat),
    SelectedAudioQuality(AudioQuality),
    SelectFolder,
    SelectFolderTextInput(String),
    SelectTab(usize),
}

struct YtGUI {
    theme: theme::Theme,
    link_state: State,
    download_folder_state: State,
    download_button_state: button::State,
    dialog_button_state: button::State,
    download_link: String,
    is_playlist: bool,
    video_format: VideoFormat,
    resolution: Resolution,
    audio_format: AudioFormat,
    audio_quality: AudioQuality,
    download_folder: Option<PathBuf>,
    placeholder: String,
    active_tab: usize,
}

impl Default for YtGUI {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            download_folder: Some(PathBuf::from("~/Videos")),
            link_state: State::default(),
            download_folder_state: State::default(),
            download_button_state: button::State::default(),
            dialog_button_state: button::State::default(),
            download_link: String::default(),
            is_playlist: bool::default(),
            video_format: VideoFormat::default(),
            resolution: Resolution::default(),
            audio_format: AudioFormat::default(),
            audio_quality: AudioQuality::default(),
            placeholder: "Download link".to_string(),
            active_tab: 0,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Resolution {
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
    Webm,
    ThreeGP,
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
    Ogg,
    Webm,
    Opus,
}

impl AudioFormat {
    const ALL: [AudioFormat; 5] = [
        AudioFormat::Mp3,
        AudioFormat::Wav,
        AudioFormat::Ogg,
        AudioFormat::Opus,
        AudioFormat::Webm,
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

impl Sandbox for YtGUI {
    type Message = Message;
    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        "Youtubedlp-GUI".to_string()
    }

    fn update(&mut self, event: Message) {
        if !self.download_folder.clone().unwrap().is_dir()
            && !self.download_folder_state.is_focused()
        {
            self.download_folder = Some(PathBuf::from("~/Videos"));
        }

        match event {
            Message::Download(link) => {
                if self.download_link.is_empty() {
                    self.placeholder = "No Download link was provided!".to_string();
                    return;
                }
                self.placeholder = "Download link".to_string();

                let mut command = Command::new("yt-dlp");
                if self.is_playlist {
                    command.args([
                        &link,
                        "-o",
                        &format!(
                            "{}/{}",
                            self.download_folder
                                .clone()
                                .unwrap()
                                .to_str()
                                .expect("No Videos Directory"),
                            "%(uploader)s/%(playlist)s - %(title)s.%(ext)s"
                        ),
                        "--yes-playlist",
                    ]);
                } else {
                    command.args([
                        &link,
                        "-o",
                        &format!(
                            "{}/{}",
                            self.download_folder
                                .clone()
                                .unwrap()
                                .to_str()
                                .expect("No Videos Directory"),
                            "%(title)s.%(ext)s"
                        ),
                    ]);
                }

                match self.active_tab {
                    0 => {
                        // Video tab
                        match self.video_format {
                            // VideoFormat::Flv => {
                            //     command.args(["--format", "flv"]);
                            // }
                            VideoFormat::Mp4 => {
                                command.args(["--format", "mp4"]);
                            }
                            VideoFormat::ThreeGP => {
                                command.args(["--format", "3gp"]);
                            }
                            VideoFormat::Webm => {
                                command.args(["--format", "webm"]);
                            }
                        }

                        match self.resolution {
                            Resolution::FourK => {
                                command.args(["-S", "res:2160"]);
                            }
                            Resolution::TwoK => {
                                command.args(["-S", "res:1440"]);
                            }
                            Resolution::FullHD => {
                                command.args(["-S", "res:1080"]);
                            }
                            Resolution::Hd => {
                                command.args(["-S", "res:720"]);
                            }
                            Resolution::Sd => {
                                command.args(["-S", "res:480"]);
                            }
                        }
                    }
                    1 => {
                        // Audio tab
                        match self.audio_format {
                            AudioFormat::Mp3 => {
                                command.args(["--format", "mp3"]);
                            }
                            AudioFormat::Wav => {
                                command.args(["--format", "wav"]);
                            }
                            AudioFormat::Ogg => {
                                command.args(["--format", "ogg"]);
                            }
                            AudioFormat::Opus => {
                                command.args(["--format", "opus"]);
                            }
                            AudioFormat::Webm => {
                                command.args(["--format", "webm"]);
                            }
                        }

                        match self.audio_quality {
                            AudioQuality::Best => {
                                command.args(["--audio-quality", "10"]);
                            }
                            AudioQuality::Good => {
                                command.args(["--audio-quality", "8"]);
                            }
                            AudioQuality::Medium => {
                                command.args(["--audio-quality", "6"]);
                            }
                            AudioQuality::Low => {
                                command.args(["--audio-quality", "4"]);
                            }
                        }
                    }
                    _ => {}
                }

                command.spawn().expect("failed to execute process");
            }
            Message::InputChanged(input) => {
                self.download_link = input;
            }
            Message::SelectedResolution(resolution) => {
                self.resolution = resolution;
            }
            Message::TogglePlaylist(is_playlist) => {
                self.is_playlist = is_playlist;
            }
            Message::SelectedVideoFormat(format) => {
                self.video_format = format;
            }
            Message::SelectFolder => {
                if let Ok(Some(path)) = FileDialog::new()
                    .set_location(
                        self.download_folder
                            .clone()
                            .expect("download folder")
                            .to_str()
                            .expect("download folder as string"),
                    )
                    .show_open_single_dir()
                {
                    self.download_folder = Some(path);
                }
            }
            Message::SelectFolderTextInput(folder_string) => {
                let path = PathBuf::from(folder_string);

                self.download_folder = Some(path);
            }
            Message::SelectTab(tab_number) => {
                self.active_tab = tab_number;
            }
            Message::SelectedAudioFormat(format) => {
                self.audio_format = format;
            }
            Message::SelectedAudioQuality(quality) => {
                self.audio_quality = quality;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content: Element<_> = Column::new()
            .push(
                Row::new()
                    .push(Text::new("Enter URL: "))
                    .push(
                        TextInput::new(
                            &mut self.link_state,
                            &self.placeholder,
                            &self.download_link,
                            Message::InputChanged,
                        )
                        .style(self.theme)
                        .size(FONT_SIZE)
                        .width(Length::Fill),
                    )
                    .push(
                        Checkbox::new(self.is_playlist, "Playlist", Message::TogglePlaylist)
                            .style(self.theme),
                    )
                    .spacing(7)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Tabs::new(self.active_tab, Message::SelectTab)
                    .push(
                        iced_aw::TabLabel::Text("Video".to_string()),
                        Column::new()
                            .push(
                                YtGUI::video_resolutions(self.resolution, self.theme)
                                    .width(Length::Fill),
                            )
                            .push(YtGUI::video_formats(self.video_format, self.theme)),
                    )
                    .push(
                        iced_aw::TabLabel::Text("Audio".to_string()),
                        Column::new()
                            .push(YtGUI::audio_qualities(self.audio_quality, self.theme))
                            .push(YtGUI::audio_formats(self.audio_format, self.theme)),
                    )
                    .height(Length::Shrink)
                    .width(Length::Units(1))
                    .tab_bar_width(Length::Units(1))
                    .tab_bar_style(self.theme),
            )
            .push(
                Row::new()
                    .push(
                        Button::new(&mut self.dialog_button_state, Text::new("Browse"))
                            .on_press(Message::SelectFolder)
                            .style(self.theme),
                    )
                    .push(
                        TextInput::new(
                            &mut self.download_folder_state,
                            "",
                            self.download_folder.clone().unwrap().to_str().unwrap(),
                            Message::SelectFolderTextInput,
                        )
                        .style(self.theme),
                    )
                    .push(
                        Button::new(&mut self.download_button_state, Text::new("Download"))
                            .on_press(Message::Download(self.download_link.clone()))
                            .style(self.theme),
                    )
                    .spacing(SPACING)
                    .align_items(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .align_items(iced::Alignment::Fill)
            .spacing(20)
            .padding(20)
            .into();

        // let content = content.explain(Color::BLACK);

        Container::new(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .style(self.theme)
            .into()
    }
}

impl YtGUI {
    fn video_resolutions(resolution: Resolution, theme: Theme) -> Row<'static, Message> {
        Row::new()
            .push(Text::new("Resolution: ").size(FONT_SIZE))
            .push(
                Radio::new(
                    Resolution::FourK,
                    "4K",
                    Some(resolution),
                    Message::SelectedResolution,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .push(
                Radio::new(
                    Resolution::TwoK,
                    "1440p",
                    Some(resolution),
                    Message::SelectedResolution,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .push(
                Radio::new(
                    Resolution::FullHD,
                    "1080p",
                    Some(resolution),
                    Message::SelectedResolution,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .push(
                Radio::new(
                    Resolution::Hd,
                    "720p",
                    Some(resolution),
                    Message::SelectedResolution,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .push(
                Radio::new(
                    Resolution::Sd,
                    "480p",
                    Some(resolution),
                    Message::SelectedResolution,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .spacing(SPACING)
            .align_items(iced::Alignment::Center)
            .padding(12)
    }

    fn video_formats(format: VideoFormat, theme: Theme) -> Row<'static, Message> {
        Row::new()
            .push(Text::new("Preferred Format: ").size(FONT_SIZE))
            .push(
                Radio::new(
                    VideoFormat::Mp4,
                    "MP4",
                    Some(format),
                    Message::SelectedVideoFormat,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .push(
                Radio::new(
                    VideoFormat::Webm,
                    "WEBM",
                    Some(format),
                    Message::SelectedVideoFormat,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .push(
                Radio::new(
                    VideoFormat::ThreeGP,
                    "3GP",
                    Some(format),
                    Message::SelectedVideoFormat,
                )
                .size(19)
                .text_size(FONT_SIZE)
                .style(theme),
            )
            .spacing(SPACING)
            .align_items(iced::Alignment::Center)
            .padding(12)
    }
    fn audio_formats(format: AudioFormat, theme: Theme) -> Row<'static, Message> {
        Row::new()
            .push(Text::new("Preferred Format: ").size(FONT_SIZE))
            .push(
                AudioFormat::ALL
                    .iter()
                    .cloned()
                    .fold(Row::new(), |row, audio_format| {
                        row.push(
                            Radio::new(
                                audio_format,
                                audio_format.to_string().to_ascii_uppercase(),
                                Some(format),
                                Message::SelectedAudioFormat,
                            )
                            .size(19)
                            .text_size(FONT_SIZE)
                            .style(theme),
                        )
                        .spacing(SPACING)
                    }),
            )
            .spacing(SPACING)
            .align_items(iced::Alignment::Center)
            .padding(12)
    }

    fn audio_qualities(quality: AudioQuality, theme: Theme) -> Row<'static, Message> {
        Row::new()
            .push(Text::new("Quality: ").size(FONT_SIZE))
            .push(
                AudioQuality::ALL
                    .iter()
                    .cloned()
                    .fold(Row::new(), |row, audio_quality| {
                        row.push(
                            Radio::new(
                                audio_quality,
                                audio_quality.to_string(),
                                Some(quality),
                                Message::SelectedAudioQuality,
                            )
                            .size(19)
                            .text_size(FONT_SIZE)
                            .style(theme),
                        )
                        .spacing(SPACING)
                    }),
            )
            .spacing(SPACING)
            .align_items(iced::Alignment::Center)
            .padding(12)
    }
}

fn main() -> iced::Result {
    let settings = Settings {
        window: window::Settings {
            size: (600, 275),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    };

    YtGUI::run(settings)
}
