use iced::border::Radius;
use iced::overlay::menu;
use iced::widget::{
    button, checkbox, container, pick_list, progress_bar, radio, scrollable, text, text_input,
};
use iced::{application, theme, Background, Border, Color};
use iced_aw::style::tab_bar;
use iced_aw::{modal, style::card};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Theme;

const SURFACE: Color = Color::from_rgb(
    0x40 as f32 / 255.0,
    0x44 as f32 / 255.0,
    0x4B as f32 / 255.0,
);

const DISABLED: Color = Color::from_rgb(
    0x30 as f32 / 255.0,
    0x34 as f32 / 255.0,
    0x3B as f32 / 255.0,
);

const PLACEHOLDER: Color = Color::from_rgb(0.4, 0.4, 0.4);

const ACCENT: Color = Color::from_rgb(
    0x6F as f32 / 255.0,
    0xFF as f32 / 255.0,
    0xE9 as f32 / 255.0,
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

pub fn ytdlp_gui_theme() -> theme::Theme {
    theme::Theme::custom(
        String::from("ytdlp_gui_theme"),
        theme::Palette {
            background: SURFACE,
            text: Color::WHITE,
            primary: ACTIVE,
            success: HOVERED,
            danger: DANGER,
        },
    )
}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> iced::application::Appearance {
        iced::application::Appearance {
            background_color: SURFACE,
            text_color: Color::WHITE,
        }
    }
}

impl card::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> iced_aw::card::Appearance {
        iced_aw::card::Appearance {
            background: SURFACE.into(),
            body_text_color: Color::WHITE,
            border_radius: 5.,
            head_background: ACTIVE.into(),
            head_text_color: Color::WHITE,
            border_color: Color::TRANSPARENT,
            close_color: Color::WHITE,
            ..Default::default()
        }
    }
}

impl modal::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> iced_aw::style::modal::Appearance {
        iced_aw::style::modal::Appearance {
            background: Color::from_rgba(0.01, 0.01, 0.01, 0.5).into(),
        }
    }
}

impl tab_bar::StyleSheet for Theme {
    type Style = ();

    fn active(&self, style: &Self::Style, is_active: bool) -> iced_aw::style::tab_bar::Appearance {
        if is_active {
            iced_aw::style::tab_bar::Appearance {
                background: Some(ACTIVE.into()),
                tab_label_background: ACTIVE.into(),
                ..self.hovered(&style, is_active)
            }
        } else {
            iced_aw::style::tab_bar::Appearance {
                background: Some(SURFACE.into()),
                tab_label_background: SURFACE.into(),
                ..self.hovered(&style, is_active)
            }
        }
    }
    fn hovered(
        &self,
        _style: &Self::Style,
        _is_active: bool,
    ) -> iced_aw::style::tab_bar::Appearance {
        iced_aw::style::tab_bar::Appearance {
            background: Some(HOVERED.into()),
            text_color: Color::WHITE,
            border_color: None,
            border_width: 0.,
            icon_color: Color::default(),
            tab_label_background: HOVERED.into(),
            tab_label_border_color: Color::TRANSPARENT,
            tab_label_border_width: 1.,
            icon_background: None,
            icon_border_radius: Radius::default(),
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb8(0x36, 0x39, 0x3F).into()),
            text_color: Color::WHITE.into(),
            ..Default::default()
        }
    }
}

impl text::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: Self::Style) -> text::Appearance {
        text::Appearance {
            color: Some(Color::WHITE),
        }
    }
}

impl radio::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style, _is_active: bool) -> radio::Appearance {
        radio::Appearance {
            background: SURFACE.into(),
            dot_color: ACTIVE,
            border_width: 1.0,
            border_color: ACTIVE,
            text_color: None,
        }
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> radio::Appearance {
        radio::Appearance {
            background: Color { a: 0.5, ..SURFACE }.into(),
            ..self.active(style, is_active)
        }
    }
}

impl text_input::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: SURFACE.into(),
            icon_color: Color::TRANSPARENT,
            border: Border {
                radius: Radius::from(2.0),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border: Border {
                width: 1.0,
                color: ACCENT,
                ..Default::default()
            },
            ..self.active(style)
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        PLACEHOLDER
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::WHITE
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        ACTIVE
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        DISABLED
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            ..self.active(style)
        }
    }
}

impl button::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(ACTIVE.into()),
            border: Border {
                radius: Radius::from(3.0),
                ..Default::default()
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(HOVERED.into()),
            text_color: Color::WHITE,
            ..self.active(style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            border: Border {
                width: 1.0,
                color: Color::WHITE,
                ..Default::default()
            },
            ..self.hovered(style)
        }
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: SURFACE.into(),
            bar: ACTIVE.into(),
            border_radius: Radius::from(5.0),
        }
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: if is_checked { ACTIVE } else { SURFACE }.into(),
            border: Border {
                radius: Radius::from(2.0),
                width: 1.0,
                color: ACTIVE,
            },
            text_color: None,
            icon_color: Color::WHITE,
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: Color {
                a: 0.8,
                ..if is_checked { ACTIVE } else { SURFACE }
            }
            .into(),
            ..self.active(style, is_checked)
        }
    }
}

impl pick_list::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: Color::WHITE,
            placeholder_color: PLACEHOLDER,
            handle_color: Color::WHITE,
            background: Background::Color(SURFACE),
            border: Border {
                radius: Radius::from(2.0),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            border: Border {
                color: HOVERED,
                ..Default::default()
            },
            background: Background::Color(HOVERED),
            ..self.active(style)
        }
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Appearance {
        iced::widget::scrollable::Appearance {
            container: container::Appearance {
                background: Some(Color::from_rgb8(0x36, 0x39, 0x3F).into()),
                text_color: Color::WHITE.into(),
                ..Default::default()
            },
            scrollbar: scrollable::Scrollbar {
                background: None,
                border: Border {
                    radius: Radius::from(2.),
                    width: 0.,
                    color: Color::TRANSPARENT,
                },
                scroller: scrollable::Scroller {
                    color: ACCENT,
                    border: Border {
                        radius: Radius::from(0.),
                        width: 0.,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> iced::widget::scrollable::Appearance {
        if is_mouse_over_scrollbar {
            iced::widget::scrollable::Appearance {
                scrollbar: scrollable::Scrollbar {
                    scroller: scrollable::Scroller {
                        color: HOVERED,
                        border: Border {
                            width: 1.,
                            ..Default::default()
                        },
                    },
                    ..self.active(style).scrollbar
                },
                ..self.active(style)
            }
        } else {
            iced::widget::scrollable::Appearance {
                scrollbar: scrollable::Scrollbar {
                    scroller: scrollable::Scroller {
                        color: HOVERED,
                        ..self.active(style).scrollbar.scroller
                    },
                    ..self.active(style).scrollbar
                },
                ..self.active(style)
            }
        }
    }
}

impl menu::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> menu::Appearance {
        menu::Appearance {
            text_color: Color::WHITE,
            background: Background::Color(ACCENT),
            border: Border {
                width: 0.,
                radius: Radius::from(0.),
                color: Color::TRANSPARENT,
            },
            selected_text_color: Color::BLACK,
            selected_background: Background::Color(ACTIVE),
        }
    }
}

impl iced_table::StyleSheet for Theme {
    type Style = ();

    fn header(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb8(0x36, 0x39, 0x3F).into()),
            text_color: Color::WHITE.into(),
            ..Default::default()
        }
    }

    fn footer(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb8(0x36, 0x39, 0x3F).into()),
            text_color: Color::WHITE.into(),
            ..Default::default()
        }
    }

    fn row(&self, style: &Self::Style, index: usize) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb8(0x36, 0x39, 0x3F).into()),
            text_color: Color::WHITE.into(),
            ..Default::default()
        }
    }

    fn divider(&self, style: &Self::Style, hovered: bool) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb8(0x36, 0x39, 0x3F).into()),
            text_color: Color::WHITE.into(),
            ..Default::default()
        }
    }
}
