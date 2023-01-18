use std::{
    path::PathBuf,
    process::Child,
    sync::{atomic::AtomicBool, Arc},
};

use iced::executor;
use iced::{
    futures::{
        channel::mpsc::{self, UnboundedSender},
        StreamExt,
    },
    widget::{button, checkbox, column, progress_bar, row, text, text_input},
    Application, Length, Subscription,
};
use iced_aw::Card;
use iced_native::subscription;

use native_dialog::FileDialog;
// use theme::widget::Element;

pub mod command;
pub mod theme;
pub mod video_options;
pub mod widgets;

use widgets::{Column, Container, Modal, Tabs};

use crate::video_options::{AudioFormat, AudioQuality, VideoFormat, VideoResolution};
use crate::{
    command::{playlist_options, ProgressState},
    video_options::Options,
};

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

pub struct YtGUI {
    download_link: String,
    is_playlist: bool,
    options: Options,
    download_folder: Option<PathBuf>,

    show_modal: bool,
    placeholder: String,
    active_tab: usize,
    ui_message: String,

    sender: Option<UnboundedSender<String>>,
    command: command::Command,
    progress: f32,
    progress_state: ProgressState,
}

impl Default for YtGUI {
    fn default() -> Self {
        Self {
            download_link: String::default(),
            is_playlist: bool::default(),
            options: Options::default(),
            download_folder: Some(PathBuf::from("~/Videos")),

            show_modal: false,
            placeholder: "Download link".to_string(),
            active_tab: 0,
            ui_message: String::default(),

            sender: None,
            command: command::Command::default(),
            progress: 0.,
            progress_state: ProgressState::Hide,
        }
    }
}

impl YtGUI {
    pub fn command_update(&mut self, message: command::Message) {
        let mut args = Vec::new();

        self.command.kill_child = Arc::new(AtomicBool::new(false));
        match message {
            command::Message::Run(link) => {
                if link.is_empty() {
                    self.placeholder = String::from("No Download link was provided!");
                    return;
                }

                self.placeholder = String::from("Download link");

                args.push(link);

                match self.active_tab {
                    0 => {
                        // Video tab

                        let mut video = String::new();
                        args.push(String::from("-S"));

                        video.push_str(self.options.video_resolution.options());

                        video.push_str(self.options.video_format.options());

                        args.push(video);
                    }
                    1 => {
                        // Audio tab

                        // Extract audio from Youtube video
                        args.push(String::from("-x"));

                        args.push(String::from("--audio-format"));
                        args.push(self.options.audio_format.options());

                        args.push(String::from("--audio-quality"));
                        args.push(self.options.audio_quality.options());
                    }
                    _ => {}
                }

                args.append(&mut playlist_options(
                    self.is_playlist,
                    self.download_folder.clone(),
                ));

                self.command.command(
                    args,
                    &mut self.show_modal,
                    &mut self.ui_message,
                    self.sender.clone(),
                );
            }
            command::Message::Stop => {
                match self
                    .command
                    .shared_child
                    .clone()
                    .expect("Shared child")
                    .kill()
                {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        println!("killed the child")
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        println!("{_e}")
                    }
                };
                self.show_modal = false;
                self.progress_state = ProgressState::Hide;
                self.progress = 0.;
                self.ui_message.clear();
            }
            command::Message::Finished => {
                match self
                    .command
                    .shared_child
                    .clone()
                    .expect("Shared child")
                    .kill()
                {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        println!("killed the child")
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        println!("{_e}")
                    }
                };
                self.progress = 0.;
                self.progress_state = ProgressState::Hide;
                if self.ui_message.contains("Already") {
                    return;
                }
                self.ui_message = String::from("Finished!");
            }
        }
    }
}

impl Application for YtGUI {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = theme::Theme;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Message>) {
        (Self::default(), iced::Command::none())
    }

    fn title(&self) -> String {
        "Youtube Downloader".to_string()
    }

    fn update(&mut self, event: Message) -> iced::Command<Message> {
        match event {
            Message::Command(message) => {
                self.command_update(message);
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
                            println!("killed the child");
                        }
                    }
                    return iced::Command::single(iced_native::command::Action::Window(
                        iced_native::window::Action::Close,
                    ));
                }
            }
        }

        iced::Command::none()
    }

    fn view(&self) -> widgets::Element<Message> {
        let content: widgets::Element<Message> = Column::new()
            .push(
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
            )
            .push(
                Tabs::new(self.active_tab, Message::SelectTab)
                    .push(
                        iced_aw::TabLabel::Text("Video".to_string()),
                        column![
                            Options::video_resolutions(self.options.video_resolution)
                                .width(Length::Fill),
                            Options::video_formats(self.options.video_format),
                        ],
                    )
                    .push(
                        iced_aw::TabLabel::Text("Audio".to_string()),
                        column![
                            Options::audio_qualities(self.options.audio_quality),
                            Options::audio_formats(self.options.audio_format),
                        ],
                    )
                    .height(Length::Shrink)
                    .width(Length::Units(1))
                    .tab_bar_width(Length::Units(1)),
            )
            .push(
                row![
                    button("Browse").on_press(Message::SelectFolder),
                    text_input(
                        "",
                        self.download_folder.clone().unwrap().to_str().unwrap(),
                        Message::SelectFolderTextInput,
                    ),
                    button(text("Download")).on_press(Message::Command(command::Message::Run(
                        self.download_link.clone(),
                    ))),
                ]
                .spacing(SPACING)
                .align_items(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .align_items(iced::Alignment::Fill)
            .spacing(20)
            .padding(20)
            .into();

        let content = Modal::new(self.show_modal, content, || {
            let progress_bar_row = row![];
            Card::new(
                text("Downloading")
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center),
                column![
                    text(self.ui_message.clone())
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
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
            .width(Length::Fill)
            .max_height(70)
            .max_width(300)
            .on_close(Message::Command(command::Message::Stop))
            .into()
        });

        // let content = content.explain(Color::BLACK);

        Container::new(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let iced_events = subscription::events().map(Message::IcedEvent);
        Subscription::batch(vec![bind(), iced_events])
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
