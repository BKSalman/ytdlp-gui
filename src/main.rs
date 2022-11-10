#![cfg_attr(not(debug_assertion), windows_subsystem = "windows")]
use iced::executor;
use iced::{
    futures::{
        channel::mpsc::{self, UnboundedSender},
        StreamExt,
    },
    widget::{
        button, checkbox, column, container, progress_bar, radio, row, text, text_input, Row,
    },
    window, Application, Element, Length, Settings, Subscription,
};

#[allow(unused_imports)]
use iced::Color;

use iced_native::subscription;
use std::path::PathBuf;
use std::process::Child;

use iced_aw::{Card, Modal, Tabs};

use native_dialog::FileDialog;

mod command;
mod theme;
mod video_options;

use video_options::{Options, VideoFormat, VideoResolution, AudioFormat, AudioQuality};

use theme::Theme;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const FONT_SIZE: u16 = 18;

const SPACING: u16 = 10;


#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    TogglePlaylist(bool),
    SelectedVideoFormat(VideoFormat),
    SelectedResolution(VideoResolution),
    SelectedAudioFormat(AudioFormat),
    SelectedAudioQuality(AudioQuality),
    SelectFolder,
    SelectFolderTextInput(String),
    SelectTab(usize),
    EventRecieved(String),
    Ready(UnboundedSender<String>),
    Command(command::Message),
    IcedEvent(iced_native::Event),
}

struct YtGUI {
    theme: iced::Theme,

    show_modal: bool,

    download_link: String,
    is_playlist: bool,
    options: Options,
    download_folder: Option<PathBuf>,

    placeholder: String,
    active_tab: usize,
    ui_message: String,
    sender: Option<UnboundedSender<String>>,
    command: command::Command,
    progress: f32,
    should_exit: bool,
    progress_state: ProgressState,
}

impl Default for YtGUI {
    fn default() -> Self {
        Self {
            theme: theme::ytdlp_gui_theme(),
            download_folder: Some(PathBuf::from("~/Videos")),
            download_link: String::default(),
            is_playlist: bool::default(),
            options: Options::default(),
            placeholder: "Download link".to_string(),
            active_tab: 0,
            show_modal: false,
            ui_message: String::default(),
            sender: None,
            command: command::Command::default(),
            progress: 0.,
            should_exit: false,
            progress_state: ProgressState::Hide,
        }
    }
}

pub enum ProgressState {
    Show,
    Hide,
}

impl Application for YtGUI {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = iced::Theme;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        (Self::default(), iced::Command::none())
    }

    fn title(&self) -> String {
        "Youtube Downloader".to_string()
    }

    fn update(&mut self, event: Message) -> iced::Command<Message> {
        match event {
            Message::Command(message) => {
                self.command.update(
                    message,
                    &mut self.show_modal,
                    &mut self.placeholder,
                    self.active_tab,
                    self.options,
                    self.is_playlist,
                    &mut self.download_folder,
                    &mut self.ui_message,
                    &mut self.progress,
                    &mut self.progress_state,
                    self.sender.clone(),
                );
            }
            Message::InputChanged(input) => {
                self.download_link = input;
            }
            Message::SelectedResolution(resolution) => {
                self.options.video_resolution = resolution;
            }
            Message::TogglePlaylist(is_playlist) => {
                self.is_playlist = is_playlist;
            }
            Message::SelectedVideoFormat(format) => {
                self.options.video_format = format;
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
                self.options.audio_format = format;
            }
            Message::SelectedAudioQuality(quality) => {
                self.options.audio_quality = quality;
            }
            Message::EventRecieved(progress) => {
                if self.progress == 100. {
                    self.ui_message = String::from("Processing...");
                    return iced::Command::none();
                }

                if progress.contains('%') {
                    self.progress_state = ProgressState::Show;

                    let words = progress
                        .split(' ')
                        .map(String::from)
                        .filter(|str| str.chars().filter(|char| char.is_numeric()).count() != 0)
                        .collect::<Vec<String>>();

                    if let Ok(percentage) = words[0].trim_end_matches('%').parse::<f32>() {
                        self.progress = percentage;
                    }

                    self.ui_message = words[1..].join(" | ");

                    return iced::Command::none();
                } else if progress.contains("[ExtractAudio]") {
                    self.ui_message = String::from("Extracting audio");
                    self.progress_state = ProgressState::Hide;
                    return iced::Command::none();
                } else if progress.contains("has already been downloaded") {
                    self.ui_message = String::from("Already downloaded");
                    self.progress_state = ProgressState::Hide;
                    return iced::Command::none();
                } else if progress.contains("Encountered a video that did not match filter") {
                    self.ui_message =
                        String::from("Playlist box needs to be checked to download a playlist");
                    self.progress_state = ProgressState::Hide;
                    return iced::Command::none();
                }
                #[cfg(debug_assertions)]
                println!("{progress}");
            }
            Message::Ready(sender) => {
                self.sender = Some(sender);
            }
            Message::IcedEvent(event) => {
                if let iced_native::Event::Window(iced_native::window::Event::CloseRequested) =
                    event
                {
                    if let Some(child) = self.command.shared_child.clone() {
                        if child.kill().is_ok() {
                            #[cfg(debug_assertions)]
                            println!("killed the child lmao");
                        }
                    }
                    self.should_exit = true;
                }
            }
        }

        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            row![
                text("Enter URL: "),
                text_input(
                    // TODO: make modal appear and notify the use they didn't enter a link
                    &self.placeholder,
                    &self.download_link,
                    Message::InputChanged,
                )
                // .style(self.theme)
                .size(FONT_SIZE)
                .width(Length::Fill),
                checkbox("Playlist", self.is_playlist, Message::TogglePlaylist) // .style(self.theme),
            ]
            .spacing(7)
            .align_items(iced::Alignment::Center),
            Tabs::new(self.active_tab, Message::SelectTab)
                .push(
                    iced_aw::TabLabel::Text("Video".to_string()),
                    column![
                        Options::video_resolutions(self.options.video_resolution).width(Length::Fill),
                        Options::video_formats(self.options.video_format)
                    ]
                )
                .push(
                    iced_aw::TabLabel::Text("Audio".to_string()),
                    column![
                        Options::audio_qualities(self.options.audio_quality),
                        Options::audio_formats(self.options.audio_format)
                    ]
                )
                .height(Length::Shrink)
                .width(Length::Units(1))
                .tab_bar_width(Length::Units(1)),
            // .tab_bar_style(self.theme),
            row![
                button("Browse").on_press(Message::SelectFolder),
                // .style(self.theme),
                text_input(
                    "",
                    self.download_folder.clone().unwrap().to_str().unwrap(),
                    Message::SelectFolderTextInput,
                ),
                // .style(self.theme),
                button(text("Download")).on_press(Message::Command(command::Message::Run(
                    self.download_link.clone(),
                ))),
                // .style(self.theme),
            ]
            .spacing(SPACING)
            .align_items(iced::Alignment::Center),
        ]
        .width(Length::Fill)
        .align_items(iced::Alignment::Fill)
        .spacing(20)
        .padding(20);

        let content = Modal::new(self.show_modal, content, || {
            let progress_bar_row = Row::new();

            Card::new(
                text("Downloading")
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center),
                column![
                    row![text(self.ui_message.clone())
                        .horizontal_alignment(iced::alignment::Horizontal::Center),]
                    .height(Length::Fill),
                    match self.progress_state {
                        ProgressState::Show => progress_bar_row
                            .push(progress_bar(0.0..=100., self.progress))
                            .height(Length::Fill),
                        ProgressState::Hide => progress_bar_row.height(Length::Units(0)),
                    }
                ]
                .align_items(iced::Alignment::Center),
            )
            // .style(self.theme)
            .width(Length::Fill)
            .max_height(70)
            .max_width(300)
            .on_close(Message::Command(command::Message::Stop))
            .into()
        });

        // let content = content.explain(Color::BLACK);

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            // .style(self.theme)
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
                                    Some(Message::Command(command::Message::Finished)),
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
