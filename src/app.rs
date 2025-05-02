use std::path::PathBuf;

use iced::widget::{
    button, checkbox, column, container, horizontal_space, pick_list, progress_bar, row, text,
    text_input,
};
use iced::{window, Event, Length, Point, Subscription};
use iced_aw::Tabs;
use url::Url;

use crate::error::DownloadError;
use crate::media_options::{playlist_options, Options};
use crate::sponsorblock::SponsorBlockOption;
use crate::theme::{pick_list_menu_style, pick_list_style, tab_bar_style};
// use crate::widgets::Tabs;
use crate::{
    choose_folder,
    progress::{parse_progress, Progress},
    Message, WindowPosition, YtGUI,
};

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
            Message::ProgressEvent(progress) => {
                if !self.command.is_running() {
                    return iced::Task::none();
                }

                match parse_progress(&progress) {
                    Ok(progress) => {
                        for progress in progress {
                            match progress {
                                Progress::Downloading {
                                    eta,
                                    downloaded_bytes,
                                    total_bytes,
                                    total_bytes_estimate,
                                    elapsed: _,
                                    speed,
                                    playlist_count,
                                    playlist_index,
                                } => {
                                    self.progress = Some(
                                        (downloaded_bytes
                                            / total_bytes
                                                .unwrap_or(total_bytes_estimate.unwrap_or(0.)))
                                            * 100.,
                                    );

                                    if let Some((playlist_count, playlist_index)) =
                                        playlist_count.zip(playlist_index)
                                    {
                                        self.playlist_progress = Some(format!(
                                            "Downloading {}/{}",
                                            playlist_index, playlist_count
                                        ));
                                    }

                                    // `eta as i64` rounds it
                                    // for examlpe: 12.368520936129604 as i64 = 12
                                    let eta = chrono::Duration::seconds(eta.unwrap_or(0.) as i64);

                                    let downloaded_megabytes = downloaded_bytes / 1024_f32.powi(2);
                                    let total_downloaded = if downloaded_megabytes > 1024. {
                                        format!("{:.2}GB", downloaded_megabytes / 1024.)
                                    } else {
                                        format!("{:.2}MB", downloaded_megabytes)
                                    };

                                    self.download_message = Some(Ok(format!(
                                                        "{total_downloaded} | {speed:.2}MB/s | ETA {eta_mins:02}:{eta_secs:02}",
                                                        speed = speed.unwrap_or(0.) / 1024_f32.powi(2),
                                                        eta_mins = eta.num_minutes(),
                                                        eta_secs = eta.num_seconds() - (eta.num_minutes() * 60),
                                                    )));
                                }
                                Progress::PostProcessing { status: _ } => {
                                    self.download_message = Some(Ok(String::from("Processing...")));
                                }
                                Progress::EndOfPlaylist => {
                                    tracing::info!("end of playlist");
                                    self.command.kill();
                                    self.progress = None;
                                    self.download_message =
                                        Some(Ok(String::from("Finished playlist!")));
                                    self.log_download();
                                }
                                Progress::EndOfVideo => {
                                    if !self.is_playlist {
                                        if self.command.is_multiple_videos() {
                                            self.command.finished_single_video();
                                        } else {
                                            self.command.kill();
                                            self.progress = None;
                                            self.download_message =
                                                Some(Ok(String::from("Finished!")));
                                            self.log_download();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        self.command.kill();
                        self.progress = None;
                        self.download_message = Some(Err(DownloadError::Progress(e)));
                        self.log_download();
                    }
                }
            }
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
                            self.window_width = size.width as f32;
                            self.window_height = size.height as f32;
                        }
                        window::Event::Moved(pos) if self.config.save_window_position => {
                            self.window_pos = Point::new(pos.x as f32, pos.y as f32);
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
                            self.show_download_message(download_message)
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
                            self.show_download_message(download_message)
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

    fn show_download_message<'a>(
        &'a self,
        download_message: &'a Result<String, DownloadError>,
    ) -> iced::widget::Column<'a, Message> {
        match download_message {
            Ok(download_message) => column![
                row![
                    text(download_message).align_x(iced::alignment::Horizontal::Center),
                    horizontal_space(),
                    text(self.playlist_progress.as_deref().unwrap_or_default()),
                    button("X").on_press(Message::StopDownload).padding([5, 25]),
                ]
                .spacing(SPACING)
                .width(iced::Length::Fill)
                .align_y(iced::Alignment::Center)
                .padding(12),
                if let Some(progress) = self.progress {
                    row![progress_bar(0.0..=100., progress)]
                        .spacing(SPACING)
                        .width(iced::Length::Fill)
                        .align_y(iced::Alignment::Center)
                        .padding(12)
                } else {
                    row![]
                }
            ]
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
            Err(e) => {
                column![
                    row![text(e.to_string()).align_x(iced::alignment::Horizontal::Center)]
                        .spacing(SPACING)
                        .width(iced::Length::Fill)
                        .align_y(iced::Alignment::Center)
                        .padding(12),
                ]
            }
        }
    }
}
