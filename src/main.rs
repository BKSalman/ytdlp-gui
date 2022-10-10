use std::{
    path::{Path, PathBuf},
    process::Command,
};

use iced::{
    button,
    text_input::{self, State},
    window, Background, Button, Checkbox, Color, Column, Container, Element, Length, Radio, Row,
    Sandbox, Settings, Text, TextInput,
};

use native_dialog::FileDialog;

#[derive(Debug, Clone)]
pub enum Message {
    Download(String),
    InputChanged(String),
    SelectedResolution(Resolution),
    TogglePlaylist(bool),
    SelectedVideoFormat(VideoFormat),
    SelectFolder,
    SelectFolderTextInput(String),
}

// #[derive(Default)]
struct YtGUI {
    link_state: State,
    download_folder_state: State,
    download_button_state: button::State,
    dialog_button_state: button::State,
    download_link: String,
    resolution: Resolution,
    is_playlist: bool,
    format: VideoFormat,
    download_folder: Option<PathBuf>,
    placeholder: String,
}

impl Default for YtGUI {
    fn default() -> Self {
        Self {
            download_folder: Some(PathBuf::from("~/Videos")),
            link_state: State::default(),
            download_folder_state: State::default(),
            download_button_state: button::State::default(),
            dialog_button_state: button::State::default(),
            download_link: String::default(),
            resolution: Resolution::default(),
            is_playlist: bool::default(),
            format: VideoFormat::default(),
            placeholder: "Download link".to_string(),
        }
    }
}

pub struct TextInputStyles;

impl text_input::StyleSheet for TextInputStyles {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::WHITE),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self) -> Color {
        Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn selection_color(&self) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
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

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoFormat {
    #[default]
    Mp4,
    Webm,
    ThreeGP,
    Flv,
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
                
                match self.format {
                    VideoFormat::Flv => {
                        command.args([
                            "--format",
                            "flv"
                        ]);
                    }
                    VideoFormat::Mp4 => {
                        command.args([
                            "--format",
                            "mp4"
                        ]);
                    }
                    VideoFormat::ThreeGP => {
                        command.args([
                            "--format",
                            "3gp"
                        ]);
                    }
                    VideoFormat::Webm => {
                        command.args([
                            "--format",
                            "webm"
                        ]);
                    }
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
                self.format = format;
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
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content: Element<_> = Column::new()
            .spacing(20)
            .padding(20)
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
                        .style(TextInputStyles)
                        .size(18)
                        .width(Length::Fill),
                    )
                    .push(Checkbox::new(
                        self.is_playlist,
                        "Playlist",
                        Message::TogglePlaylist,
                    ))
                    .spacing(7)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Row::new()
                    .push(Text::new("Resolution: "))
                    .push(
                        Radio::new(
                            Resolution::FourK,
                            "4K",
                            Some(self.resolution),
                            Message::SelectedResolution,
                        )
                        .size(19),
                    )
                    .push(
                        Radio::new(
                            Resolution::TwoK,
                            "1440p",
                            Some(self.resolution),
                            Message::SelectedResolution,
                        )
                        .size(19),
                    )
                    .push(
                        Radio::new(
                            Resolution::FullHD,
                            "1080p",
                            Some(self.resolution),
                            Message::SelectedResolution,
                        )
                        .size(19),
                    )
                    .push(
                        Radio::new(
                            Resolution::Hd,
                            "720p",
                            Some(self.resolution),
                            Message::SelectedResolution,
                        )
                        .size(19),
                    )
                    .push(
                        Radio::new(
                            Resolution::Sd,
                            "480p",
                            Some(self.resolution),
                            Message::SelectedResolution,
                        )
                        .size(19),
                    )
                    .spacing(12)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Row::new()
                    .push(Text::new("Preferred Format: "))
                    .push(
                        Radio::new(
                            VideoFormat::Mp4,
                            "MP4",
                            Some(self.format),
                            Message::SelectedVideoFormat,
                        )
                        .size(19),
                    )
                    .push(
                        Radio::new(
                            VideoFormat::Webm,
                            "WEBM",
                            Some(self.format),
                            Message::SelectedVideoFormat,
                        )
                        .size(19),
                    )
                    .push(
                        Radio::new(
                            VideoFormat::ThreeGP,
                            "3GP",
                            Some(self.format),
                            Message::SelectedVideoFormat,
                        )
                        .size(19),
                    )
                    // .push(
                    //     Radio::new(
                    //         VideoFormat::Flv,
                    //         "FLV",
                    //         Some(self.format),
                    //         Message::SelectedVideoFormat,
                    //     )
                    //     .size(19),
                    // )
                    .spacing(12)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Row::new()
                    .push(
                        Button::new(&mut self.dialog_button_state, Text::new("Browse"))
                            .on_press(Message::SelectFolder),
                    )
                    .push(
                        TextInput::new(
                            &mut self.download_folder_state,
                            "",
                            self.download_folder.clone().unwrap().to_str().unwrap(),
                            Message::SelectFolderTextInput,
                        )
                        .style(TextInputStyles),
                    )
                    .push(
                        Button::new(&mut self.download_button_state, Text::new("Download"))
                            .on_press(Message::Download(self.download_link.clone())),
                    )
                    .spacing(12)
                    .align_items(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .align_items(iced::Alignment::Fill)
            .into();

        // let content = content.explain(Color::BLACK);

        Container::new(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .into()
    }
}

fn main() -> iced::Result {
    let settings = Settings {
        window: window::Settings {
            size: (600, 240),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    };

    YtGUI::run(settings)
}
