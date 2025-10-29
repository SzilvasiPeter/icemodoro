//! Manages application settings, including themes, timer durations, and keyboard shortcuts.

use std::collections::btree_map::ValuesMut;

use super::persistence;

use iced::widget::{button, column, container, horizontal_rule, radio, row, scrollable, text};
use iced::{Element, Length, Theme};
use iced_aw::widget::number_input;

use serde::{Deserialize, Serialize};

/// List of all available themes and their display names, used in UI rendering.
const ALL_THEMES: [(&str, AppTheme); 10] = [
    ("Catppuccin Frappe", AppTheme::CatppuccinFrappe),
    ("Catppuccin Latte", AppTheme::CatppuccinLatte),
    ("Dark", AppTheme::Dark),
    ("Light", AppTheme::Light),
    ("Gruvbox Dark", AppTheme::GruvboxDark),
    ("Gruvbox Light", AppTheme::GruvboxLight),
    ("Solarized Dark", AppTheme::SolarizedDark),
    ("Solarized Light", AppTheme::SolarizedLight),
    ("TokyoNight Storm", AppTheme::TokyoNightStorm),
    ("TokyoNight Light", AppTheme::TokyoNightLight),
];

/// Defines the color themes available in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    CatppuccinFrappe,
    CatppuccinLatte,
    Dark,
    Light,
    GruvboxDark,
    GruvboxLight,
    SolarizedDark,
    SolarizedLight,
    TokyoNightStorm,
    TokyoNightLight,
}

impl AppTheme {
    /// Converts the custom `AppTheme` into the corresponding `iced::Theme`.
    pub fn to_iced_theme(self) -> Theme {
        match self {
            Self::CatppuccinFrappe => Theme::CatppuccinFrappe,
            Self::CatppuccinLatte => Theme::CatppuccinLatte,
            Self::Dark => Theme::Dark,
            Self::Light => Theme::Light,
            Self::GruvboxDark => Theme::GruvboxDark,
            Self::GruvboxLight => Theme::GruvboxLight,
            Self::SolarizedDark => Theme::SolarizedDark,
            Self::SolarizedLight => Theme::SolarizedLight,
            Self::TokyoNightStorm => Theme::TokyoNightStorm,
            Self::TokyoNightLight => Theme::TokyoNightLight,
        }
    }
}

/// Indicates which session type a setting applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Pomodoro,
    Break,
    LongBreak,
}

/// Stores user-configurable settings for session durations and themes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Settings {
    pub work_min: u8,
    pub break_min: u8,
    pub long_break_min: u8,
    pub work_theme: AppTheme,
    pub break_theme: AppTheme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_min: 25,
            break_min: 5,
            long_break_min: 60,
            work_theme: AppTheme::SolarizedDark,
            break_theme: AppTheme::SolarizedLight,
        }
    }
}

/// Messages used for updating the settings tab.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    PomodoroChanged(u8),
    BreakChanged(u8),
    LongBreakChanged(u8),
    ThemeChanged(SessionType, AppTheme),
    Submit,
}

impl Settings {
    /// Loads saved settings from persistent storage or returns default.
    pub fn new() -> Self {
        persistence::load("settings.json").unwrap_or_default()
    }

    /// Processes messages and updates the component's state.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::PomodoroChanged(value) => self.work_min = value,
            Message::BreakChanged(value) => self.break_min = value,
            Message::LongBreakChanged(value) => self.long_break_min = value,
            Message::ThemeChanged(session, theme) => match session {
                SessionType::Pomodoro => self.work_theme = theme,
                SessionType::Break => self.break_theme = theme,
                SessionType::LongBreak => self.break_theme = theme,
            },
            Message::Submit => {
                let _ = persistence::save("settings.json", &self);
            }
        }
    }

    /// Builds the main view for the Settings tab.
    pub fn view(&self) -> Element<'_, Message> {
        scrollable(
            column![
                self.view_timer_settings(),
                self.view_theme_settings(),
                Self::view_shortcuts(),
                button("Apply Settings").on_press(Message::Submit),
            ]
            .spacing(20)
            .padding(10),
        )
        .into()
    }

    /// View section for configuring session timer durations.
    fn view_timer_settings(&self) -> Element<'_, Message> {
        column![
            text("Time (minutes)").size(20),
            horizontal_rule(1),
            row![
                column![
                    text("Pomodoro"),
                    number_input(&self.work_min, 1..=240, Message::PomodoroChanged)
                ],
                column![
                    text("Break"),
                    number_input(&self.break_min, 1..=60, Message::BreakChanged)
                ],
                column![
                    text("long break"),
                    number_input(&self.break_min, 1..=60, Message::LongBreakChanged)
                ]
            ]
            .spacing(20),
        ]
        .spacing(10)
        .into()
    }

    /// View section for configuring color themes for both sessions.
    fn view_theme_settings(&self) -> Element<'_, Message> {
        // Helper function to generate a column of theme radio buttons.
        let theme_radios = |session, active_theme| {
            column(ALL_THEMES.iter().map(|(name, app_theme)| {
                // Create a small colored swatch showing theme's background color
                let color_swatch_style = |_: &Theme| -> container::Style {
                    let palette = *app_theme.to_iced_theme().extended_palette();
                    container::Style {
                        background: Some(palette.background.strong.color.into()),
                        ..Default::default()
                    }
                };
                let color_swatch = container(text(""))
                    .width(Length::Fixed(10.0))
                    .height(Length::Fixed(10.0))
                    .style(color_swatch_style);

                let radio = radio(*name, *app_theme, Some(active_theme), move |theme| {
                    Message::ThemeChanged(session, theme)
                });

                row![color_swatch, radio]
                    .spacing(10)
                    .align_y(iced::Alignment::Center)
                    .into()
            }))
            .spacing(5)
        };

        column![
            text("Color Themes").size(20),
            horizontal_rule(1),
            row![
                column![
                    text("Pomodoro"),
                    theme_radios(SessionType::Pomodoro, self.work_theme)
                ]
                .spacing(10),
                column![
                    text("Break"),
                    theme_radios(SessionType::Break, self.break_theme)
                ]
                .spacing(10)
            ]
            .spacing(40)
        ]
        .spacing(10)
        .into()
    }

    /// View section listing all keyboard shortcuts available in the app.
    fn view_shortcuts<'a>() -> Element<'a, Message> {
        let shortcut_row = |key, desc| -> iced::widget::Row<'a, Message> {
            row![text(key).font(iced::Font::MONOSPACE).width(200), text(desc)].spacing(10)
        };

        let content = column![
            text("Shortcuts"),
            horizontal_rule(1),
            shortcut_row("Space", "Start/Stop timer"),
            shortcut_row("r", "Reset timer"),
            shortcut_row("f", "Finish session"),
            shortcut_row("n", "Focus new task input"),
            shortcut_row("a", "Activate/Deactivate first task"),
            shortcut_row("↑ / ↓", "Navigate active task"),
            shortcut_row("e", "Edit active task"),
            shortcut_row("s", "Complete active task"),
            shortcut_row("d", "Delete active task"),
            shortcut_row("x", "End day"),
            shortcut_row("Ctrl + Tab", "Next tab"),
            shortcut_row("Shift + Tab", "Previous tab"),
        ]
        .spacing(10);

        container(content).center_x(Length::Fill).into()
    }
}
