use iced::{
    widget::{button, container, responsive, scrollable, text},
    Length,
};
use iced_table::table;
use std::path::PathBuf;

use crate::{theme, widgets};

pub struct QueueVideo {
    url: String,
    download_path: PathBuf,
}

impl QueueVideo {
    pub fn new(url: String, download_path: PathBuf) -> Self {
        Self { url, download_path }
    }
}

pub struct QueueList {
    columns: Vec<Column>,
    rows: Vec<QueueVideo>,
    header: scrollable::Id,
    body: scrollable::Id,
    footer: scrollable::Id,
}

impl QueueList {
    pub fn new(queue: Vec<QueueVideo>) -> Self {
        let columns = vec![
            Column::new(ColumnKind::Index, 20.),
            Column::new(ColumnKind::URL, 50.),
            Column::new(ColumnKind::DownloadPath, 200.),
            Column::new(ColumnKind::DownloadButton, 100.),
        ];

        Self {
            rows: queue,
            columns,
            header: scrollable::Id::unique(),
            body: scrollable::Id::unique(),
            footer: scrollable::Id::unique(),
        }
    }

    pub fn view(&self) -> widgets::Element<Message> {
        responsive::<Message, theme::Theme, iced::Renderer>(|size| {
            let mut table = table::table(
                self.header.clone(),
                self.body.clone(),
                &self.columns,
                &self.rows,
                Message::SyncHeader,
            );

            // if self.resize_columns_enabled {
            //     table = table.on_column_resize(Message::Resizing, Message::Resized);
            // }
            // if self.footer_enabled {
            //     table = table.footer(self.footer.clone());
            // }
            // if self.min_width_enabled {
            //     table = table.min_width(size.width);
            // }

            table.into()
        })
        .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Command<crate::Message> {
        match message {
            Message::SyncHeader(offset) => {
                return iced::Command::batch(vec![
                    scrollable::scroll_to(self.header.clone(), offset),
                    scrollable::scroll_to(self.footer.clone(), offset),
                ])
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SyncHeader(scrollable::AbsoluteOffset),
}

impl From<Message> for crate::Message {
    fn from(value: Message) -> Self {
        crate::Message::QueueList(value)
    }
}

struct Column {
    kind: ColumnKind,
    width: f32,
    resize_offset: Option<f32>,
}

impl Column {
    fn new(kind: ColumnKind, width: f32) -> Self {
        Self {
            kind,
            width,
            resize_offset: None,
        }
    }
}

enum ColumnKind {
    Index,
    URL,
    DownloadPath,
    DownloadButton,
}

impl<'a> table::Column<'a, Message, theme::Theme, iced::Renderer> for Column {
    type Row = QueueVideo;

    fn header(
        &'a self,
        _col_index: usize,
    ) -> iced_runtime::core::Element<'a, Message, theme::Theme, iced::Renderer> {
        let content = match self.kind {
            ColumnKind::Index => "i",
            ColumnKind::URL => "URL",
            ColumnKind::DownloadPath => "Download path",
            ColumnKind::DownloadButton => "",
        };

        container(text(content)).height(24).center_y().into()
    }

    fn cell(
        &'a self,
        _col_index: usize,
        row_index: usize,
        row: &'a Self::Row,
    ) -> widgets::Element<Message> {
        let content: widgets::Element<Message> = match self.kind {
            ColumnKind::Index => text(row_index).into(),
            ColumnKind::URL => text(row.url.clone()).into(),
            ColumnKind::DownloadPath => text(row.download_path.display()).into(),
            ColumnKind::DownloadButton => button("Download").into(),
        };

        container(content)
            .width(Length::Fill)
            .height(32)
            .center_y()
            .into()
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        self.resize_offset
    }
}
