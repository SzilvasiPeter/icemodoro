use super::persistence;

use iced::widget::{button, column, container, horizontal_rule, radio, row, scrollable, text};
use iced::{Element, Length, Theme};
use iced_aw::widget::number_input;

use serde::{Deserialize, Serialize};

// TODO: Add documentation
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
    pub fn to_iced_theme(self) -> Theme {
        match self {
            AppTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            AppTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            AppTheme::Dark => Theme::Dark,
            AppTheme::Light => Theme::Light,
            AppTheme::GruvboxDark => Theme::GruvboxDark,
            AppTheme::GruvboxLight => Theme::GruvboxLight,
            AppTheme::SolarizedDark => Theme::SolarizedDark,
            AppTheme::SolarizedLight => Theme::SolarizedLight,
            AppTheme::TokyoNightStorm => Theme::TokyoNightStorm,
            AppTheme::TokyoNightLight => Theme::TokyoNightLight,
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Pomodoro,
    Break,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Settings {
    pub work_min: u8,
    pub break_min: u8,
    pub work_theme: AppTheme,
    pub break_theme: AppTheme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_min: 25,
            break_min: 5,
            work_theme: AppTheme::SolarizedDark,
            break_theme: AppTheme::SolarizedLight,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    PomodoroChanged(u8),
    BreakChanged(u8),
    ThemeChanged(SessionType, AppTheme),
    Submit,
}

impl Settings {
    pub fn new() -> Self {
        persistence::load("settings.json").unwrap_or_default()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::PomodoroChanged(value) => self.work_min = value,
            Message::BreakChanged(value) => self.break_min = value,
            Message::ThemeChanged(session, theme) => match session {
                SessionType::Pomodoro => self.work_theme = theme,
                SessionType::Break => self.break_theme = theme,
            },
            Message::Submit => {
                persistence::save("settings.json", &self).ok();
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        scrollable(
            column![
                self.view_timer_settings(),
                self.view_theme_settings(),
                self.view_shortcuts(),
                button("Apply Settings").on_press(Message::Submit),
            ]
            .spacing(20)
            .padding(10),
        )
        .into()
    }

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
                ]
            ]
            .spacing(20),
        ]
        .spacing(10)
        .into()
    }

    fn view_theme_settings(&self) -> Element<'_, Message> {
        let theme_radios = |session, active_theme| {
            column(
                ALL_THEMES
                    .iter()
                    .map(|(name, theme)| {
                        let color_swatch_style = |_: &Theme| -> container::Style {
                            let palette = theme.to_iced_theme().extended_palette().clone();
                            container::Style {
                                background: Some(palette.background.strong.color.into()),
                                ..Default::default()
                            }
                        };
                        let color_swatch = container(text(""))
                            .width(Length::Fixed(10.0))
                            .height(Length::Fixed(10.0))
                            .style(color_swatch_style);

                        let radio = radio(*name, *theme, Some(active_theme), move |theme| {
                            Message::ThemeChanged(session, theme)
                        });

                        row![color_swatch, radio]
                            .spacing(10)
                            .align_y(iced::Alignment::Center)
                            .into()
                    })
                    .collect::<Vec<Element<_>>>(),
            )
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

    fn shortcut_row<'a>(&self, key: &'a str, desc: &'a str) -> iced::widget::Row<'a, Message> {
        row![text(key).font(iced::Font::MONOSPACE).width(150), text(desc)].spacing(10)
    }

    fn view_shortcuts<'a>(&self) -> Element<'a, Message> {
        let content = column![
            text("Shortcuts"),
            horizontal_rule(1),
            self.shortcut_row("Space", "Start/Stop timer"),
            self.shortcut_row("r", "Reset timer"),
            self.shortcut_row("f", "Finish session"),
            self.shortcut_row("n", "Focus new task input"),
            self.shortcut_row("a", "Activate/Deactivate first task"),
            self.shortcut_row("↑ / ↓", "Navigate active task"),
            self.shortcut_row("s", "Complete active task"),
            self.shortcut_row("e", "Edit active task"),
            self.shortcut_row("d", "Delete active task"),
            self.shortcut_row("x", "End day"),
            self.shortcut_row("Tab", "Next tab"),
            self.shortcut_row("Shift + Tab", "Previous tab"),
        ]
        .spacing(10);

        container(content).center_x(Length::Fill).into()
    }
}
