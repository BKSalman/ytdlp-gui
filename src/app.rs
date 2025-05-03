use std::path::PathBuf;

use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{window, Event, Length, Point, Subscription};
use iced_aw::Tabs;
use url::Url;

use crate::error::DownloadError;
use crate::media_options::{playlist_options, Options};
use crate::sponsorblock::SponsorBlockOption;
use crate::theme::{pick_list_menu_style, pick_list_style, tab_bar_style};
// use crate::widgets::Tabs;
use crate::{choose_folder, Message, WindowPosition, YtGUI};

pub const FONT_SIZE: u16 = 18;

pub const SPACING: u16 = 10;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Tab {
    Video,
    Audio,
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
                if !self.is_choosing_folder {
                    self.is_choosing_folder = true;

                    return iced::Task::perform(
                        choose_folder(
                            self.config
                                .download_folder
                                .clone()
                                .unwrap_or_else(|| "~/Videos".into()),
                        ),
                        Message::SelectedDownloadFolder,
                    );
                }
            }
            Message::SelectedDownloadFolder(folder) => {
                if let Some(path) = folder {
                    self.config.download_folder = Some(path);
                }
                self.is_choosing_folder = false;
            }
            Message::SelectFolderTextInput(folder_string) => {
                let path = PathBuf::from(folder_string);

                self.config.download_folder = Some(path);
            }
            Message::SelectTab(selected_tab) => {
                self.active_tab = selected_tab;
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
                        _ => {}
                    }
                }
            }
            Message::None => {}
            Message::FontLoaded(_) => {
                // focus download link text input
                return iced::widget::text_input::focus(self.download_text_input_id.clone());
            }
            Message::StartDownload(link) => {
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

                match self.active_tab {
                    Tab::Video => {
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
                    Tab::Audio => {
                        // Audio tab

                        // Extract audio from Youtube video
                        args.push("-x");

                        args.push("--audio-format");
                        args.push(self.config.options.audio_format.options());

                        args.push("--audio-quality");
                        args.push(self.config.options.audio_quality.options());
                    }
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
                    self.config.bin_dir.clone(),
                    self.sender.clone(),
                    links_num,
                );
            }
            Message::StopDownload => {
                self.command.kill();
                let _ = self.progress.take();
                let _ = self.download_message.take();
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
                    iced_aw::TabLabel::Text("Video".to_string()),
                    column![row![
                        if let Some(download_message) = &self.download_message {
                            self.show_download_progress(download_message)
                        } else {
                            column![
                                Options::video_resolutions(self.config.options.video_resolution),
                                Options::video_formats(self.config.options.video_format),
                            ]
                            .width(Length::Fill)
                        }
                    ]]
                    .width(Length::Fill),
                )
                .push(
                    Tab::Audio,
                    iced_aw::TabLabel::Text("Audio".to_string()),
                    column![row![
                        if let Some(download_message) = &self.download_message {
                            self.show_download_progress(download_message)
                        } else {
                            column![
                                Options::audio_qualities(self.config.options.audio_quality),
                                Options::audio_formats(self.config.options.audio_format),
                            ]
                        }
                    ]],
                )
                .set_active_tab(&self.active_tab)
                .height(Length::Shrink)
                .width(Length::FillPortion(1))
                .tab_bar_width(Length::FillPortion(1))
                .tab_bar_style(tab_bar_style),
            row![
                text_input(
                    "",
                    &self
                        .config
                        .download_folder
                        .clone()
                        .unwrap_or_else(|| "~/Videos".into())
                        .to_string_lossy()
                )
                .on_input(Message::SelectFolderTextInput),
                button("Browse").on_press(Message::SelectDownloadFolder),
            ]
            .spacing(SPACING)
            .align_y(iced::Alignment::Center),
            row![if self.progress.is_none() {
                button("Download").on_press(Message::StartDownload(self.download_link.clone()))
            } else {
                button("Download")
            }]
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
            .center_y(Length::Fill)
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
