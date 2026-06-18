use iced::overlay::menu;
use iced::widget::{self, pick_list};
use iced::{Background, Border, Color, Element, Shadow, Theme, theme};
use iced_aw::tab_bar;

use crate::{Message, YtGUI};

const BACKGROUND: Color =
    Color::from_rgb(0x36 as f32 / 255., 0x39 as f32 / 255., 0x3F as f32 / 255.);

const SURFACE: Color = Color::from_rgb(
    0x40 as f32 / 255.0,
    0x44 as f32 / 255.0,
    0x4B as f32 / 255.0,
);

const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);

const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);

const DANGER: Color = Color::from_rgb(
    0x7b as f32 / 255.0,
    0x67 as f32 / 255.0,
    0xC4 as f32 / 255.0,
);

pub fn ytdlp_gui_theme(_state: &YtGUI) -> theme::Theme {
    theme::Theme::custom(
        String::from("ytdlp_gui_theme"),
        theme::Palette {
            background: BACKGROUND,
            text: Color::WHITE,
            primary: ACTIVE,
            success: HOVERED,
            danger: DANGER,
            warning: Color::from_rgb(
                0xb7 as f32 / 255.0,
                0x7e as f32 / 255.0,
                0x33 as f32 / 255.0,
            ),
        },
    )
}

pub fn tab_bar_style(theme: &Theme, status: tab_bar::Status) -> tab_bar::Style {
    let mut base = tab_bar::tab_bar::primary(theme, status);

    match status {
        tab_bar::Status::Disabled => {
            base.tab_label_background = Background::Color(SURFACE);
        }
        _ => {}
    }

    base
}

pub fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.extended_palette();

    let active = pick_list::Style {
        text_color: Color::WHITE,
        background: SURFACE.into(),
        placeholder_color: palette.background.strong.color,
        handle_color: Color::WHITE,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
    };

    match status {
        pick_list::Status::Active => active,
        pick_list::Status::Hovered | pick_list::Status::Opened { .. } => pick_list::Style {
            border: Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
    }
}

pub fn pick_list_menu_style(theme: &Theme) -> menu::Style {
    let palette = theme.extended_palette();

    menu::Style {
        background: SURFACE.into(),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: palette.background.strong.color,
        },
        text_color: Color::WHITE,
        selected_text_color: palette.primary.strong.text,
        selected_background: palette.primary.strong.color.into(),
        shadow: Shadow::default(),
    }
}

pub fn button<'a>(content: impl Into<Element<'a, Message>>) -> widget::Button<'a, Message> {
    widget::button(content).style(|theme: &Theme, status| {
        let mut style = widget::button::primary(theme, status);
        style.text_color = theme.extended_palette().background.base.text;

        style
    })
}
