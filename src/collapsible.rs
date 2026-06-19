use iced::{
    Alignment, Element,
    widget::{Column, column, text},
};

use crate::{Message, i18n::is_rtl, theme::button};

pub fn collapsible<'a>(
    title: String,
    expanded: bool,
    on_toggle: Message,
    body: impl Into<Element<'a, Message>>,
) -> Column<'a, Message> {
    let arrow = if expanded {
        "▼"
    } else {
        if is_rtl() { "◀" } else { "▶" }
    };
    let mut title = text(format!("{arrow}  {title}"));
    if is_rtl() {
        title = title.align_x(Alignment::End);
    }

    let header = button(title).on_press(on_toggle).width(iced::Length::Fill);

    let mut col = column![header].spacing(8);

    if is_rtl() {
        col = col.align_x(Alignment::End);
    }

    if expanded {
        col = col.push(body);
    }
    col
}
