use iced::{
    Element,
    widget::{Column, column, text},
};

use crate::{Message, theme::button};

pub fn collapsible<'a>(
    title: String,
    expanded: bool,
    on_toggle: Message,
    body: impl Into<Element<'a, Message>>,
) -> Column<'a, Message> {
    let arrow = if expanded { "▼" } else { "▶" };
    let header = button(text(format!("{arrow}  {title}")))
        .on_press(on_toggle)
        .width(iced::Length::Fill);

    let mut col = column![header].spacing(8);
    if expanded {
        col = col.push(body);
    }
    col
}
