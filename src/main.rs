use std::{path::{Path, PathBuf}, process::Command};

use iced::{
    button, text_input::State, window, Button, Checkbox, Color, Column, Container, Element, Length,
    Radio, Row, Sandbox, Settings, Text, TextInput,
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
}

#[derive(Default)]
struct YtGUI {
    link_state: State,
    download_button_state: button::State,
    dialog_button_state: button::State,
    download_link: String,
    resolution: Resolution,
    is_playlist: bool,
    format: VideoFormat,
    download_folder: Option<PathBuf>,
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
        match event {
            Message::Download(link) => {
                let output_path = Path::new("~/Videos");

                Command::new("./yt-dlp")
                    .args([
                        &link,
                        "-o",
                        &format!(
                            "{}/{}",
                            output_path.to_str().expect("No Videos Directory"),
                            "%(title)s.%(ext)s"
                        ),
                    ])
                    .spawn()
                    .expect("failed to execute process");
                println!("{link}");
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
                self.download_folder = FileDialog::new()
                    .set_location("~/Downloads")
                    .show_open_single_dir()
                    .unwrap_or_else(|e| {
                        println!("{e}");
                        Some(PathBuf::from("~/Downloads"))
                    });
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
                            "Download link",
                            &self.download_link,
                            Message::InputChanged,
                        )
                        .size(25),
                    )
                    .push(Checkbox::new(
                        self.is_playlist,
                        "Playlist",
                        Message::TogglePlaylist,
                    ))
                    .spacing(12)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Row::new()
                    .push(Text::new("Resolution: "))
                    .push(Radio::new(
                        Resolution::FourK,
                        "4K",
                        Some(self.resolution),
                        Message::SelectedResolution,
                    ))
                    .push(Radio::new(
                        Resolution::TwoK,
                        "1440p",
                        Some(self.resolution),
                        Message::SelectedResolution,
                    ))
                    .push(Radio::new(
                        Resolution::FullHD,
                        "1080p",
                        Some(self.resolution),
                        Message::SelectedResolution,
                    ))
                    .push(Radio::new(
                        Resolution::Hd,
                        "720p",
                        Some(self.resolution),
                        Message::SelectedResolution,
                    ))
                    .push(Radio::new(
                        Resolution::Sd,
                        "480p",
                        Some(self.resolution),
                        Message::SelectedResolution,
                    ))
                    .spacing(12)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Row::new()
                    .push(Text::new("Preferred Format: "))
                    .push(Radio::new(
                        VideoFormat::Mp4,
                        "MP4",
                        Some(self.format),
                        Message::SelectedVideoFormat,
                    ))
                    .push(Radio::new(
                        VideoFormat::Webm,
                        "WEBM",
                        Some(self.format),
                        Message::SelectedVideoFormat,
                    ))
                    .push(Radio::new(
                        VideoFormat::ThreeGP,
                        "3GP",
                        Some(self.format),
                        Message::SelectedVideoFormat,
                    ))
                    .push(Radio::new(
                        VideoFormat::Flv,
                        "FLV",
                        Some(self.format),
                        Message::SelectedVideoFormat,
                    ))
                    .spacing(12)
                    .align_items(iced::Alignment::Center),
            )
            .push(
                Row::new().push(
                    Button::new(&mut self.dialog_button_state, Text::new("Browse"))
                        .on_press(Message::SelectFolder),
                    )
                .push(
                    Button::new(&mut self.download_button_state, Text::new("Download"))
                        .on_press(Message::Download(self.download_link.clone())),
                )
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
