use iced::widget::{button, container, scrollable, text, text_input};
use iced::{Background, Border, Color, Theme as IcedTheme};

// Colors matching the original wlaunch dark theme
pub const BACKGROUND: Color = Color::from_rgb(0.118, 0.118, 0.118); // #1e1e1e
pub const SURFACE: Color = Color::from_rgb(0.157, 0.157, 0.157); // #282828
pub const ACCENT: Color = Color::from_rgb(0.8, 0.4, 0.2); // #cc6633
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.933, 0.933, 0.933); // #eeeeee
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.6, 0.6, 0.6); // #999999
pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.25); // #404040
pub const SELECTED: Color = Color::from_rgb(0.8, 0.4, 0.2); // #cc6633
pub const HOVER: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333333

pub struct Theme;

impl Theme {
    pub fn custom() -> IcedTheme {
        IcedTheme::custom(
            "WLaunch".to_string(),
            iced::theme::Palette {
                background: BACKGROUND,
                text: TEXT_PRIMARY,
                primary: ACCENT,
                success: Color::from_rgb(0.4, 0.8, 0.4),
                danger: Color::from_rgb(0.8, 0.3, 0.3),
            },
        )
    }
}

// Container styles
pub fn main_container(theme: &IcedTheme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(BACKGROUND)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn search_container(theme: &IcedTheme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub fn results_container(theme: &IcedTheme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(BACKGROUND)),
        ..Default::default()
    }
}

pub fn details_container(theme: &IcedTheme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

// Button styles
pub fn item_button(theme: &IcedTheme, selected: bool) -> button::Style {
    let _ = theme;
    if selected {
        button::Style {
            background: Some(Background::Color(SELECTED)),
            text_color: TEXT_PRIMARY,
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    } else {
        button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub fn item_button_hover(theme: &IcedTheme) -> button::Style {
    let _ = theme;
    button::Style {
        background: Some(Background::Color(HOVER)),
        text_color: TEXT_PRIMARY,
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// Text input style
pub fn search_input(theme: &IcedTheme, _status: text_input::Status) -> text_input::Style {
    let _ = theme;
    text_input::Style {
        background: Background::Color(SURFACE),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: TEXT_SECONDARY,
        placeholder: TEXT_SECONDARY,
        value: TEXT_PRIMARY,
        selection: ACCENT,
    }
}

// Scrollable style
pub fn scrollable_style(theme: &IcedTheme, _status: scrollable::Status) -> scrollable::Style {
    let _ = theme;
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(SURFACE)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BORDER,
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(SURFACE)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BORDER,
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        },
        gap: None,
    }
}

// Text styles
pub fn primary_text(_theme: &IcedTheme) -> text::Style {
    text::Style {
        color: Some(TEXT_PRIMARY),
    }
}

pub fn secondary_text(_theme: &IcedTheme) -> text::Style {
    text::Style {
        color: Some(TEXT_SECONDARY),
    }
}

pub fn accent_text(_theme: &IcedTheme) -> text::Style {
    text::Style {
        color: Some(ACCENT),
    }
}
