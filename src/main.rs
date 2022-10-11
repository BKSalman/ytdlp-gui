use iced::{
    button, executor,
    futures::{
        channel::mpsc::{self, UnboundedSender},
        StreamExt,
    },
    text_input::State,
    window, Application, Button, Checkbox, Column, Container, Element, Length, Radio, Row,
    Settings, Subscription, Text, TextInput,
};

#[allow(unused_imports)]
use iced::Color;

use iced_native::subscription;
use std::{
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use iced_aw::{modal, Card, Modal, Tabs};

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
    EventRecieved(String),
    Ready(UnboundedSender<String>),
    CloseModal,
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
    args: Vec<String>,
    modal_state: modal::State<ModalState>,
    output: String,
    sender: Option<UnboundedSender<String>>,
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
            args: Vec::new(),
            modal_state: modal::State::default(),
            output: String::default(),
            sender: None,
        }
    }
}

#[derive(Debug, Default)]
struct ModalState;

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

impl Application for YtGUI {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        (Self::default(), iced::Command::none())
    }

    fn title(&self) -> String {
        "Youtube Downloader".to_string()
    }

    fn update(&mut self, event: Message) -> iced::Command<Message> {
        if !self.download_folder.clone().unwrap().is_dir()
            && !self.download_folder_state.is_focused()
        {
            self.download_folder = Some(PathBuf::from("~/Videos"));
        }

        match event {
            Message::Download(link) => {
                if self.download_link.is_empty() {
                    self.placeholder = "No Download link was provided!".to_string();
                    return iced::Command::none();
                }

                self.placeholder = "Download link".to_string();

                self.args.push(link);

                match self.active_tab {
                    0 => {
                        let mut video = String::new();
                        self.args.push("-S".to_string());

                        match self.resolution {
                            Resolution::FourK => {
                                video.push_str("res:2160,");
                            }
                            Resolution::TwoK => {
                                video.push_str("res:1440,");
                            }
                            Resolution::FullHD => {
                                video.push_str("res:1080,");
                            }
                            Resolution::Hd => {
                                video.push_str("res:720,");
                            }
                            Resolution::Sd => {
                                video.push_str("res:480,");
                            }
                        }

                        match self.video_format {
                            VideoFormat::Mp4 => {
                                video.push_str("ext:mp4");
                            }
                            VideoFormat::ThreeGP => {
                                video.push_str("ext:3gp");
                            }
                            VideoFormat::Webm => {
                                video.push_str("ext:webm");
                            }
                        }
                        self.args.push(video);
                    }
                    1 => {
                        // Audio tab
                        match self.audio_format {
                            AudioFormat::Mp3 => {
                                self.args.push("--format mp3".to_string());
                            }
                            AudioFormat::Wav => {
                                self.args.push("--format wav".to_string());
                            }
                            AudioFormat::Ogg => {
                                self.args.push("--format ogg".to_string());
                            }
                            AudioFormat::Opus => {
                                self.args.push("--format opus".to_string());
                            }
                            AudioFormat::Webm => {
                                self.args.push("--format webm".to_string());
                            }
                        }

                        match self.audio_quality {
                            AudioQuality::Best => {
                                self.args.push("--audio-quality 10".to_string());
                            }
                            AudioQuality::Good => {
                                self.args.push("--audio-quality 8".to_string());
                            }
                            AudioQuality::Medium => {
                                self.args.push("--audio-quality 6".to_string());
                            }
                            AudioQuality::Low => {
                                self.args.push("--audio-quality 4".to_string());
                            }
                        }
                    }
                    _ => {}
                }

                if self.is_playlist {
                    self.args.push(format!(
                        "--yes-playlist -o {}/%(uploader)s/%(playlist)s - %(title)s.%(ext)s",
                        self.download_folder
                            .clone()
                            .unwrap()
                            .to_str()
                            .expect("No Videos Directory")
                    ))
                } else {
                    println!("not");
                    self.args.push("-P".to_string());
                    self.args.push(
                        self.download_folder
                            .clone()
                            .unwrap()
                            .to_str()
                            .expect("No Videos Directory")
                            .to_string(),
                    );
                    self.args.push("-o".to_string());
                    self.args.push("%(title)s.%(ext)s".to_string())
                }

                if let Ok(command) = Command::new("yt-dlp")
                    .args(self.args.clone())
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    if let Some(stdout) = command.stdout {
                        self.modal_state.show(true);
                        let sender = Arc::new(Mutex::new(self.sender.clone().unwrap()));
                        std::thread::spawn(move || {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines().filter_map(|line| line.ok()) {
                                (*sender.lock().unwrap()).unbounded_send(line).unwrap();
                            }
                        });
                    }
                }
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
            Message::EventRecieved(progress) => {
                println!("{}", &progress);
                if progress.ends_with("has already been downloaded") {
                    self.output = "has already been downloaded".to_string();
                    return iced::Command::none();
                }
                self.output = progress;
            }
            Message::Ready(sender) => {
                self.sender = Some(sender);
            }
            Message::CloseModal => {
                self.modal_state.show(false);
            }
        }

        iced::Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let content: Element<_> = Column::new()
            .push(
                Row::new()
                    .push(Text::new("Enter URL: "))
                    .push(
                        TextInput::new(
                            &mut self.link_state,
                            // TODO: make modal appear and notify the use they didn't enter a link
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

        let content: Element<_> = Modal::new(&mut self.modal_state, content, |_state| {
            Card::new(
                Text::new("My modal"),
                Row::new()
                    .push(Text::new(self.output.clone()))
                    .align_items(iced::Alignment::Center),
            )
            .style(self.theme)
            .max_width(300)
            .on_close(Message::CloseModal)
            .into()
        }).into();

        // let content = content.explain(Color::BLACK);

        Container::new(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .style(self.theme)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        bind()
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

enum MyState {
    Starting,
    Ready(mpsc::UnboundedReceiver<String>),
}

pub fn bind() -> Subscription<Message> {
    struct Progress;

    subscription::unfold(
        std::any::TypeId::of::<Progress>(),
        MyState::Starting,
        |state| async move {
            match state {
                MyState::Starting => {
                    let (sender, receiver) = mpsc::unbounded();

                    (Some(Message::Ready(sender)), MyState::Ready(receiver))
                }
                MyState::Ready(mut progress_receiver) => {
                    let received = progress_receiver.next().await;
                    match received {
                        Some(progress) => (
                            Some(Message::EventRecieved(progress)),
                            MyState::Ready(progress_receiver),
                        ),
                        None => (None, MyState::Ready(progress_receiver)),
                    }
                }
            }
        },
    )
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
