use std::path::PathBuf;

use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{window, Event, Length, Point, Subscription};
use iced_aw::Tabs;
use url::Url;

use crate::error::DownloadError;
use crate::media_options::{playlist_options, Options};
use crate::sponsorblock::SponsorBlockOption;
use crate::theme::{pick_list_menu_style, pick_list_style, tab_bar_style};
// use crate::widgets::Tabs;
use crate::fl;
use crate::{choose_file, choose_folder, Message, WindowPosition, YtGUI};

pub const FONT_SIZE: u16 = 18;

pub const SPACING: u16 = 10;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DownloadType {
    Video,
    Audio,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Tab {
    Video,
    Audio,
    Settings,
}

impl YtGUI {
    pub fn update(&mut self, event: Message) -> iced::Task<Message> {
        match event {
            Message::InputChanged(input) => {
                self.download_link = input;
            }
            Message::SelectedResolution(resolution) => {
                self.config.options.video_resolution = resolution;
            }
            Message::TogglePlaylist(is_playlist) => {
                self.is_playlist = is_playlist;
            }
            Message::SelectedSponsorBlockOption(sponsorblock) => {
                self.sponsorblock = Some(sponsorblock);
            }
            Message::SelectedVideoFormat(format) => {
                self.config.options.video_format = format;
            }
            Message::SelectDownloadFolder => {
                if !self.is_file_dialog_open {
                    self.is_file_dialog_open = true;

                    return iced::Task::perform(
                        choose_folder(self.config.download_folder.clone()),
                        Message::SelectedDownloadFolder,
                    );
                }
            }
            Message::SelectedDownloadFolder(folder) => {
                if let Some(path) = folder {
                    self.config.download_folder = path;
                }
                self.is_file_dialog_open = false;
            }
            Message::DownloadFolderTextInput(folder_string) => {
                let path = PathBuf::from(&folder_string.to_string());

                self.config.download_folder = path;
            }
            Message::SelectDownloadFolderTextInput => {
                let path = PathBuf::from(
                    shellexpand::tilde(&self.config.download_folder.display().to_string())
                        .to_string(),
                );

                self.config.download_folder = path;
            }
            Message::SelectTab(selected_tab) => {
                self.active_tab = selected_tab;
                match self.active_tab {
                    Tab::Video => self.download_type = DownloadType::Video,
                    Tab::Audio => self.download_type = DownloadType::Audio,
                    _ => {}
                }
            }
            Message::SelectedAudioFormat(format) => {
                self.config.options.audio_format = format;
            }
            Message::SelectedAudioQuality(quality) => {
                self.config.options.audio_quality = quality;
            }
            Message::ProgressEvent(progress) => self.handle_progress_event(&progress),
            Message::IcedEvent(event) => {
                if let Event::Window(window_event) = event {
                    match window_event {
                        window::Event::CloseRequested => {
                            self.command.kill();
                            self.config.window_position = Some(WindowPosition {
                                x: self.window_pos.x,
                                y: self.window_pos.y,
                            });
                            if let Err(e) = self.config.update_config_file() {
                                tracing::error!("Failed to update config file: {e}");
                            }
                            return window::get_latest().and_then(window::close);
                        }
                        window::Event::Resized(size) => {
                            self.window_width = size.width;
                            self.window_height = size.height;
                        }
                        window::Event::Moved(pos) if self.config.save_window_position => {
                            self.window_pos = Point::new(pos.x, pos.y);
                        }
                        window::Event::Opened {
                            position: _,
                            size: _,
                        } => {
                            return iced::widget::text_input::focus(
                                self.download_text_input_id.clone(),
                            );
                        }
                        _ => {}
                    }
                }
            }
            Message::StartDownload(link) => {
                self.config.download_folder = PathBuf::from(
                    shellexpand::tilde(&self.config.download_folder.display().to_string())
                        .to_string(),
                );

                if !self.config.download_folder.exists() {
                    self.progress = None;
                    self.download_message = Some(Err(DownloadError::DownloadDir(
                        self.config.download_folder.clone(),
                    )));
                    return iced::Task::none();
                }

                let mut args: Vec<&str> = Vec::new();

                let mut links_num = 0;

                for (i, link) in link.trim().split(' ').enumerate() {
                    if Url::parse(link).is_err() {
                        self.progress = None;
                        self.download_message = Some(Err(DownloadError::InvalidURL(i + 1)));
                        return iced::Task::none();
                    }

                    self.config
                        .update_config_file()
                        .expect("update config file");

                    if link.is_empty() {
                        self.progress = None;
                        self.download_message = Some(Err(DownloadError::NoDownloadURL));
                        return iced::Task::none();
                    }

                    args.push(link);

                    links_num = i + 1;
                }

                match self.download_type {
                    DownloadType::Video => {
                        args.push("-S");

                        args.push(self.config.options.video_resolution.options());

                        // after downloading a video with a specific format
                        // yt-dlp sometimes downloads the audio and video seprately
                        // then merge them in a different format
                        // this enforces the chosen format by the user
                        args.push("--remux-video");

                        args.push(self.config.options.video_format.options());

                        tracing::info!("{args:#?}");
                    }
                    DownloadType::Audio => {
                        // Audio tab

                        // Extract audio from Youtube video
                        args.push("-x");

                        args.push("--audio-format");
                        args.push(self.config.options.audio_format.options());

                        args.push("--audio-quality");
                        args.push(self.config.options.audio_quality.options());
                    }
                }

                if let Some(cookies_file) = &self.config.cookies_file {
                    args.push("--cookies");
                    args.push(cookies_file.to_str().unwrap());
                }

                let playlist_options =
                    playlist_options(self.is_playlist, self.config.download_folder.clone());

                args.append(&mut playlist_options.iter().map(|s| &**s).collect());

                if let Some(sponsorblock) = &self.sponsorblock {
                    match sponsorblock {
                        SponsorBlockOption::Remove => {
                            args.push("--sponsorblock-remove=default");
                        }
                        SponsorBlockOption::Mark => {
                            args.push("--sponsorblock-mark=default");
                        }
                    }
                }

                self.download_message = self.command.start(
                    args,
                    self.config.bin_path.clone(),
                    self.sender.clone(),
                    links_num,
                );
            }
            Message::StopDownload => {
                self.command.kill();
                let _ = self.progress.take();
                let _ = self.download_message.take();
            }
            Message::ToggleSaveWindowPosition(save_window_position) => {
                self.config.save_window_position = save_window_position;
            }
            Message::SelectYtDlpBinPath => {
                if !self.is_file_dialog_open {
                    self.is_file_dialog_open = true;

                    return iced::Task::perform(
                        choose_file(self.config.bin_path.clone().unwrap_or("~".into())),
                        Message::SelectedYtDlpBinPath,
                    );
                }
            }
            Message::SelectedYtDlpBinPath(file) => {
                if let Some(path) = file {
                    self.config.bin_path = Some(path);
                }
                self.is_file_dialog_open = false;
            }
            Message::SelectYtDlpBitPathTextInput(file_string) => {
                let path = PathBuf::from(file_string);

                self.config.bin_path = Some(path);
            }
            Message::SelectCookiesFile => {
                if !self.is_file_dialog_open {
                    self.is_file_dialog_open = true;

                    return iced::Task::perform(
                        choose_file(self.config.cookies_file.clone().unwrap_or("~".into())),
                        Message::SelectedCookiesFile,
                    );
                }
            }
            Message::SelectedCookiesFile(file) => {
                if let Some(path) = file {
                    self.config.cookies_file = Some(path);
                }
                self.is_file_dialog_open = false;
            }
            Message::SelectCookiesFileTextInput(cookies_string) => {
                let path = PathBuf::from(cookies_string);

                self.config.cookies_file = Some(path);
            }
        }

        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<Message> {
        let content: iced::Element<Message> = column![
            row![text_input("Download link", &self.download_link)
                .on_input(Message::InputChanged)
                .on_submit(Message::StartDownload(self.download_link.clone(),))
                .size(FONT_SIZE)
                .width(Length::Fill)
                .id(self.download_text_input_id.clone()),]
            .spacing(7)
            .align_y(iced::Alignment::Center),
            row![
                row![
                    text("SponsorBlock:"),
                    pick_list(
                        vec![SponsorBlockOption::Remove, SponsorBlockOption::Mark,],
                        self.sponsorblock,
                        Message::SelectedSponsorBlockOption
                    )
                    .style(pick_list_style)
                    .menu_style(pick_list_menu_style)
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
                checkbox("Playlist", self.is_playlist).on_toggle(Message::TogglePlaylist),
            ]
            .spacing(7)
            .align_y(iced::Alignment::Center),
            Tabs::new(Message::SelectTab)
                .push(
                    Tab::Video,
                    iced_aw::TabLabel::Text(fl!("video")),
                    column![
                        row![if let Some(download_message) = &self.download_message {
                            self.show_download_progress(download_message)
                        } else {
                            column![
                                Options::video_resolutions(self.config.options.video_resolution),
                                Options::video_formats(self.config.options.video_format),
                            ]
                            .width(Length::Fill)
                        }],
                        column![
                            row![
                                text_input(
                                    "Destination path (leave blank for current working directory)",
                                    &self.config.download_folder.display().to_string()
                                )
                                .on_input(Message::DownloadFolderTextInput)
                                .on_submit(Message::SelectDownloadFolderTextInput),
                                button("Browse").on_press(Message::SelectDownloadFolder),
                            ]
                            .spacing(SPACING)
                            .align_y(iced::Alignment::Center),
                            row![if !self.command.is_running() {
                                button(text(fl!("download")))
                                    .on_press(Message::StartDownload(self.download_link.clone()))
                            } else {
                                button(text(fl!("download")))
                            }]
                        ]
                        .width(Length::Fill)
                        .align_x(iced::Alignment::Center)
                        .spacing(20)
                        .padding(20)
                    ]
                    .width(Length::Fill),
                )
                .push(
                    Tab::Audio,
                    iced_aw::TabLabel::Text(fl!("audio")),
                    column![
                        row![if let Some(download_message) = &self.download_message {
                            self.show_download_progress(download_message)
                        } else {
                            column![
                                Options::audio_qualities(self.config.options.audio_quality),
                                Options::audio_formats(self.config.options.audio_format),
                            ]
                        }],
                        column![
                            row![
                                text_input(
                                    "Destination path (leave blank for current working directory)",
                                    &self.config.download_folder.display().to_string()
                                )
                                .on_input(Message::DownloadFolderTextInput)
                                .on_submit(Message::SelectDownloadFolderTextInput),
                                button("Browse").on_press(Message::SelectDownloadFolder),
                            ]
                            .spacing(SPACING)
                            .align_y(iced::Alignment::Center),
                            row![if !self.command.is_running() {
                                button("Download")
                                    .on_press(Message::StartDownload(self.download_link.clone()))
                            } else {
                                button("Download")
                            }]
                        ]
                        .width(Length::Fill)
                        .align_x(iced::Alignment::Center)
                        .spacing(20)
                        .padding(20)
                    ],
                )
                .push(
                    Tab::Settings,
                    iced_aw::TabLabel::Text(fl!("settings")),
                    scrollable(
                        column![
                            row![checkbox(
                                "Save window position",
                                self.config.save_window_position
                            )
                            .on_toggle(Message::ToggleSaveWindowPosition)],
                            row![
                                text("yt-dlp binary path:"),
                                text_input(
                                    "Keep empty to use the system-wide binary",
                                    &self
                                        .config
                                        .bin_path
                                        .clone()
                                        .unwrap_or("".into())
                                        .to_string_lossy()
                                )
                                .on_input(Message::SelectYtDlpBitPathTextInput),
                                button("Browse").on_press(Message::SelectYtDlpBinPath),
                            ]
                            .spacing(SPACING)
                            .align_y(iced::Alignment::Center),
                            row![
                                text("Cookies file: "),
                                text_input(
                                    "",
                                    &self
                                        .config
                                        .cookies_file
                                        .clone()
                                        .unwrap_or("".into())
                                        .to_string_lossy()
                                )
                                .on_input(Message::SelectCookiesFileTextInput),
                                button("Browse").on_press(Message::SelectCookiesFile),
                            ]
                            .spacing(SPACING)
                            .align_y(iced::Alignment::Center),
                        ]
                        .width(Length::Fill)
                        .spacing(20)
                        .padding(20)
                    )
                )
                .set_active_tab(&self.active_tab)
                .height(Length::Shrink)
                .width(Length::FillPortion(1))
                .tab_bar_width(Length::FillPortion(1))
                .tab_bar_style(tab_bar_style),
        ]
        .width(Length::Fill)
        .align_x(iced::Alignment::Center)
        .spacing(20)
        .padding(20)
        .into();

        #[cfg(feature = "explain")]
        let content: crate::widgets::Element<Message> = content.into();
        #[cfg(feature = "explain")]
        let content: crate::widgets::Element<Message> = content.explain(Color::BLACK);

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(Message::IcedEvent)
    }

    pub fn end_download(&mut self, download_message: Option<Result<String, DownloadError>>) {
        self.command.kill();
        self.progress = None;
        self.download_message = download_message;
        self.log_download();
    }
}
