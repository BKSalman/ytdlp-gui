use crate::{theme::Theme, Tab};

pub type Element<'a, Message> = iced::Element<'a, Message, Theme>;

pub type Column<'a, Message> = iced::widget::Column<'a, Message, Theme>;

pub type Row<'a, Message> = iced::widget::Row<'a, Message, Theme>;

pub type Text<'a> = iced::widget::Text<'a, Theme>;

pub type TextInput<'a> = iced::widget::TextInput<'a, Theme>;

pub type Container<'a, Message> = iced::widget::Container<'a, Message, Theme>;

pub type Modal<'a, Message> = iced_aw::Modal<'a, Message, Theme>;

pub type Card<'a, Message> = iced_aw::Card<'a, Message, Theme>;

pub type Tabs<'a, Message> = iced_aw::Tabs<'a, Message, Tab, Theme>;
