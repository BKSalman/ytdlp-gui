use iced::widget::{
    button, checkbox, container, progress_bar, radio, rule, scrollable, slider, text, text_input,
    toggler,
};
use iced::{application, theme, Color};
use iced_aw::{modal, style::card, tabs};

#[derive(Debug, Clone, Copy, Default)]
pub struct Theme;

const SURFACE: Color = Color::from_rgb(
    0x40 as f32 / 255.0,
    0x44 as f32 / 255.0,
    0x4B as f32 / 255.0,
);

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
    theme::Theme::custom(theme::Palette {
        background: SURFACE,
        text: Color::WHITE,
        primary: ACTIVE,
        success: HOVERED,
        danger: DANGER,
    })
}

pub mod widget {
    use crate::theme::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
    pub type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
}
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Theme {
//     Light,
//     Dark,
// }

// impl Theme {
//     pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
// }

// impl Default for Theme {
//     fn default() -> Theme {
//         Theme::Dark
//     }
// }

// impl<'a> From<Theme> for Box<dyn card::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Card.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn modal::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Modal.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn tabs::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Tabs.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn container::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Container.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn radio::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Radio.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn text_input::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::TextInput.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn button::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => light::Button.into(),
//             Theme::Dark => dark::Button.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn scrollable::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Scrollable.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn slider::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Slider.into(),
//         }
//     }
// }

// impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::ProgressBar.into(),
//         }
//     }
// }

// impl<'a> From<Theme> for Box<dyn checkbox::StyleSheet + 'a> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Checkbox.into(),
//         }
//     }
// }

// impl From<Theme> for Box<dyn toggler::StyleSheet> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Toggler.into(),
//         }
//     }
// }

// impl From<Theme> for Box<dyn rule::StyleSheet> {
//     fn from(theme: Theme) -> Self {
//         match theme {
//             Theme::Light => Default::default(),
//             Theme::Dark => dark::Rule.into(),
//         }
//     }
// }

// mod dark {
//     use iced::{
//         button, checkbox, container, progress_bar, radio, rule, scrollable, slider, text_input,
//         toggler, Color,
//     };

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

    fn active(&self, _style: Self::Style) -> iced_aw::card::Appearance {
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

    fn active(&self, _style: Self::Style) -> iced_aw::style::modal::Appearance {
        iced_aw::style::modal::Appearance {
            background: ACTIVE.into(),
        }
    }
}

impl tabs::StyleSheet for Theme {
    type Style = ();

    fn active(&self, style: Self::Style, is_active: bool) -> iced_aw::style::tab_bar::Appearance {
        if is_active {
            iced_aw::style::tab_bar::Appearance {
                background: ACTIVE.into(),
                tab_label_background: ACTIVE.into(),
                ..self.hovered(style, is_active)
            }
        } else {
            iced_aw::style::tab_bar::Appearance {
                background: SURFACE.into(),
                tab_label_background: SURFACE.into(),
                ..self.hovered(style, is_active)
            }
        }
    }
    fn hovered(
        &self,
        _style: Self::Style,
        _is_active: bool,
    ) -> iced_aw::style::tab_bar::Appearance {
        iced_aw::style::tab_bar::Appearance {
            background: HOVERED.into(),
            text_color: Color::WHITE,
            border_color: None,
            border_width: 0.,
            icon_color: Color::default(),
            tab_label_background: HOVERED.into(),
            tab_label_border_color: Color::TRANSPARENT,
            tab_label_border_width: 1.,
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Color::from_rgb8(0x36, 0x39, 0x3F).into(),
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

// impl radio::StyleSheet for Theme {
//     type Style = ();

//     fn active(&self, _style: &Self::Style, is_active: bool) -> radio::Appearance {
//         radio::Appearance {
//             background: SURFACE.into(),
//             dot_color: ACTIVE,
//             border_width: 1.0,
//             border_color: ACTIVE,
//             text_color: None,
//         }
//     }

//     fn hovered(&self, style: &Self::Style, is_active: bool) -> radio::Appearance {
//         radio::Appearance {
//             background: Color { a: 0.5, ..SURFACE }.into(),
//             ..self.active(&style, is_active)
//         }
//     }
// }

//     impl text_input::StyleSheet for TextInput {
//         fn active(&self) -> text_input::Style {
//             text_input::Style {
//                 background: SURFACE.into(),
//                 border_radius: 2.0,
//                 border_width: 0.0,
//                 border_color: Color::TRANSPARENT,
//             }
//         }

//         fn focused(&self) -> text_input::Style {
//             text_input::Style {
//                 border_width: 1.0,
//                 border_color: ACCENT,
//                 ..self.active()
//             }
//         }

//         fn hovered(&self) -> text_input::Style {
//             text_input::Style {
//                 border_width: 1.0,
//                 border_color: Color { a: 0.3, ..ACCENT },
//                 ..self.focused()
//             }
//         }

//         fn placeholder_color(&self) -> Color {
//             Color::from_rgb(0.4, 0.4, 0.4)
//         }

//         fn value_color(&self) -> Color {
//             Color::WHITE
//         }

//         fn selection_color(&self) -> Color {
//             ACTIVE
//         }
//     }

//     pub struct Button;

impl button::StyleSheet for Theme {
    type Style = ();

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: ACTIVE.into(),
            border_radius: 3.0,
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: HOVERED.into(),
            text_color: Color::WHITE,
            ..self.active(&style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            border_width: 1.0,
            border_color: Color::WHITE,
            ..self.hovered(&style)
        }
    }
}

//     pub struct Scrollable;

//     impl scrollable::StyleSheet for Scrollable {
//         fn active(&self) -> scrollable::Scrollbar {
//             scrollable::Scrollbar {
//                 background: SURFACE.into(),
//                 border_radius: 2.0,
//                 border_width: 0.0,
//                 border_color: Color::TRANSPARENT,
//                 scroller: scrollable::Scroller {
//                     color: ACTIVE,
//                     border_radius: 2.0,
//                     border_width: 0.0,
//                     border_color: Color::TRANSPARENT,
//                 },
//             }
//         }

//         fn hovered(&self) -> scrollable::Scrollbar {
//             let active = self.active();

//             scrollable::Scrollbar {
//                 background: Color { a: 0.5, ..SURFACE }.into(),
//                 scroller: scrollable::Scroller {
//                     color: HOVERED,
//                     ..active.scroller
//                 },
//                 ..active
//             }
//         }

//         fn dragging(&self) -> scrollable::Scrollbar {
//             let hovered = self.hovered();

//             scrollable::Scrollbar {
//                 scroller: scrollable::Scroller {
//                     color: Color::from_rgb(0.85, 0.85, 0.85),
//                     ..hovered.scroller
//                 },
//                 ..hovered
//             }
//         }
//     }

//     pub struct Slider;

//     impl slider::StyleSheet for Slider {
//         fn active(&self) -> slider::Style {
//             slider::Style {
//                 rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
//                 handle: slider::Handle {
//                     shape: slider::HandleShape::Circle { radius: 9.0 },
//                     color: ACTIVE,
//                     border_width: 0.0,
//                     border_color: Color::TRANSPARENT,
//                 },
//             }
//         }

//         fn hovered(&self) -> slider::Style {
//             let active = self.active();

//             slider::Style {
//                 handle: slider::Handle {
//                     color: HOVERED,
//                     ..active.handle
//                 },
//                 ..active
//             }
//         }

//         fn dragging(&self) -> slider::Style {
//             let active = self.active();

//             slider::Style {
//                 handle: slider::Handle {
//                     color: Color::from_rgb(0.85, 0.85, 0.85),
//                     ..active.handle
//                 },
//                 ..active
//             }
//         }
//     }

//     pub struct ProgressBar;

//     impl progress_bar::StyleSheet for ProgressBar {
//         fn style(&self) -> progress_bar::Style {
//             progress_bar::Style {
//                 background: SURFACE.into(),
//                 bar: ACTIVE.into(),
//                 border_radius: 10.0,
//             }
//         }
//     }

//     pub struct Checkbox;

//     impl checkbox::StyleSheet for Checkbox {
//         fn active(&self, is_checked: bool) -> checkbox::Style {
//             checkbox::Style {
//                 background: if is_checked { ACTIVE } else { SURFACE }.into(),
//                 checkmark_color: Color::WHITE,
//                 border_radius: 2.0,
//                 border_width: 1.0,
//                 border_color: ACTIVE,
//                 text_color: None,
//             }
//         }

//         fn hovered(&self, is_checked: bool) -> checkbox::Style {
//             checkbox::Style {
//                 background: Color {
//                     a: 0.8,
//                     ..if is_checked { ACTIVE } else { SURFACE }
//                 }
//                 .into(),
//                 ..self.active(is_checked)
//             }
//         }
//     }

//     pub struct Toggler;

//     impl toggler::StyleSheet for Toggler {
//         fn active(&self, is_active: bool) -> toggler::Style {
//             toggler::Style {
//                 background: if is_active { ACTIVE } else { SURFACE },
//                 background_border: None,
//                 foreground: if is_active { Color::WHITE } else { ACTIVE },
//                 foreground_border: None,
//             }
//         }

//         fn hovered(&self, is_active: bool) -> toggler::Style {
//             toggler::Style {
//                 background: if is_active { ACTIVE } else { SURFACE },
//                 background_border: None,
//                 foreground: if is_active {
//                     Color {
//                         a: 0.5,
//                         ..Color::WHITE
//                     }
//                 } else {
//                     Color { a: 0.5, ..ACTIVE }
//                 },
//                 foreground_border: None,
//             }
//         }
//     }

//     pub struct Rule;

//     impl rule::StyleSheet for Rule {
//         fn style(&self) -> rule::Style {
//             rule::Style {
//                 color: SURFACE,
//                 width: 2,
//                 radius: 1.0,
//                 fill_mode: rule::FillMode::Padded(15),
//             }
//         }
//     }
// }

// // pub struct TextInputStyles;

// // impl text_input::StyleSheet for TextInputStyles {
// //     fn active(&self) -> text_input::Style {
// //         text_input::Style {
// //             background: Background::Color(Color::WHITE),
// //             border_radius: 2.0,
// //             border_width: 1.0,
// //             border_color: Color::from_rgb(0.7, 0.7, 0.7),
// //         }
// //     }

// //     fn focused(&self) -> text_input::Style {
// //         text_input::Style {
// //             border_color: Color::from_rgb(0.5, 0.5, 0.5),
// //             ..self.active()
// //         }
// //     }

// //     fn placeholder_color(&self) -> Color {
// //         Color::from_rgb(0.7, 0.7, 0.7)
// //     }

// //     fn value_color(&self) -> Color {
// //         Color::from_rgb(0.3, 0.3, 0.3)
// //     }

// //     fn selection_color(&self) -> Color {
// //         Color::from_rgb(0.8, 0.8, 1.0)
// //     }
// // }
