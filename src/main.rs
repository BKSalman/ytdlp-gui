#![cfg_attr(not(debug_assertion), windows_subsystem = "windows")]
use iced::{
    button, executor,
    futures::{
        channel::mpsc::{self, UnboundedSender},
        StreamExt,
    },
    text_input::State,
    window, Application, Button, Checkbox, Column, Container, Element, Length, ProgressBar, Radio,
    Row, Settings, Subscription, Text, TextInput,
};

#[allow(unused_imports)]
use iced::Color;

use iced_native::subscription;
use std::path::PathBuf;
use std::process::Child;

use iced_aw::{modal, Card, Modal, Tabs};

use native_dialog::FileDialog;

use strum::Display;

mod command;
mod theme;

use theme::Theme;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const FONT_SIZE: u16 = 18;

const SPACING: u16 = 10;

const RADIO_DOT_SIZE: u16 = 15;

#[derive(Debug, Clone)]
pub enum Message {
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
    Command(command::Message),
    IcedEvent(iced_native::Event)
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
    modal_state: modal::State<ModalState>,
    output: String,
    sender: Option<UnboundedSender<String>>,
    command: command::Command,
    progress: f32,
    should_exit: bool,
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
            modal_state: modal::State::default(),
            output: String::default(),
            sender: None,
            command: command::Command::default(),
            progress: 0.,
            should_exit: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct ModalState;

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
            Message::Command(message) => {
                self.command.update(
                    message,
                    &mut self.modal_state,
                    &mut self.placeholder,
                    self.active_tab,
                    self.resolution,
                    self.video_format,
                    self.audio_format,
                    self.audio_quality,
                    self.is_playlist,
                    &mut self.download_folder,
                    &mut self.output,
                    &mut self.progress,
                    self.sender.clone(),
                );
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
                if progress.contains("%") {
                    let words = progress
                        .split(' ')
                        .map(String::from)
                        .filter(|str| str.chars().filter(|char| char.is_numeric()).count() != 0)
                        .collect::<Vec<String>>();

                    if let Ok(percentage) = words[0].trim_end_matches("%").parse::<f32>() {
                        self.progress = percentage;
                    }
                    
                    if self.progress == 100. {
                        self.output = String::from("Processing...");
                        return iced::Command::none();
                    }
                    
                    self.output = words[1..].join(" | ");

                    return iced::Command::none();
                } else if progress.contains("[ExtractAudio]") {
                    self.output = "Extracting Audio".to_string();
                    return iced::Command::none();

                } else if progress.ends_with("has already been downloaded") {
                    self.output = "has already been downloaded".to_string();
                    return iced::Command::none();
                }
                self.output = progress;
            }
            Message::Ready(sender) => {
                self.sender = Some(sender);
            }
            Message::IcedEvent(event) => {
                match event {
                    iced_native::Event::Window(iced_native::window::Event::CloseRequested)=> {
                        if let Some(child) = self.command.shared_child.clone() {
                            if child.kill().is_ok() {
                                #[cfg(debug_assertions)]
                                println!("killed the child lmao");
                            }
                        }
                        self.should_exit = true;
                    }
                    _ => {}
                }
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
                            .on_press(Message::Command(command::Message::Run(
                                self.download_link.clone(),
                            )))
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
                Text::new("Progress"),
                Column::new()
                    .push(
                        Row::new()
                            .height(Length::FillPortion(1))
                            .push(Text::new(self.output.clone())),
                    )
                    .push(
                        Row::new()
                            .height(Length::FillPortion(1))
                            .push(ProgressBar::new(0.0..=100., self.progress)),
                    )
                    .align_items(iced::Alignment::Center),
            )
            .style(self.theme)
            .max_height(100)
            .max_width(300)
            .on_close(Message::Command(command::Message::Stop))
            .into()
        })
        .into();

        // let content = content.explain(Color::BLACK);

        Container::new(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .style(self.theme)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let iced_events = subscription::events().map(Message::IcedEvent);
        Subscription::batch(vec![bind(), iced_events])
    }
    
    fn should_exit(&self) -> bool {
        self.should_exit
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
                .size(RADIO_DOT_SIZE)
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
                .size(RADIO_DOT_SIZE)
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
                .size(RADIO_DOT_SIZE)
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
                .size(RADIO_DOT_SIZE)
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
                .size(RADIO_DOT_SIZE)
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
                .size(RADIO_DOT_SIZE)
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
                .size(RADIO_DOT_SIZE)
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
                .size(RADIO_DOT_SIZE)
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
                            .size(RADIO_DOT_SIZE)
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
                            .size(RADIO_DOT_SIZE)
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
                        Some(progress) => {
                            if progress.contains("Finished") {
                                (
                                    Some(Message::Command(command::Message::Stop)),
                                    MyState::Ready(progress_receiver),
                                )
                            } else {
                                (
                                    Some(Message::EventRecieved(progress)),
                                    MyState::Ready(progress_receiver),
                                )
                            }
                        }
                        None => (None, MyState::Ready(progress_receiver)),
                    }
                }
            }
        },
    )
}

pub enum ChildMessage {
    Ready(UnboundedSender<Child>),
    ChildEvent(Child),
}

pub enum ChildState {
    Starting,
    Ready(mpsc::UnboundedReceiver<Child>),
}

fn main() -> iced::Result {
    
    let settings = Settings {
        id: Some("ytdlp-gui".to_string()),
        window: window::Settings {
            size: (600, 275),
            resizable: false,
            ..Default::default()
        },
        exit_on_close_request: false,
        ..Default::default()
    };

    YtGUI::run(settings)
}
