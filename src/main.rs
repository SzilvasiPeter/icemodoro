#![forbid(unsafe_code)]
//! # Icemodoro
//!
//! A simple Pomodoro and To-Do list application built with the Iced GUI library.
//! The application follows the Elm architecture, where the state is updated via messages,
//! and the view displays the UI interface from the current state.

mod persistence;
mod pomodoro;
mod report;
mod setting;

use iced::keyboard::key::{Key, Named};
use iced::widget::{column, container, image, row, text};
use iced::{Element, Length, Subscription, Theme};
use iced_aw::{TabLabel, Tabs};

use pomodoro::Pomodoro;
use report::Report;
use setting::Settings;

fn main() -> iced::Result {
    iced::application("Icemodoro", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(iced_aw::iced_fonts::REQUIRED_FONT_BYTES)
        .run()
}

/// Holds the entire state of the application, including the state for each tab.
struct App {
    active_tab: TabId,
    pomodoro: Pomodoro,
    settings: Settings,
    report: Report,
}

/// Defines all messages that can update the application's state.
///
/// It wraps messages from child components (e.g. `pomodoro`, `settings`) to enable central processing in `App::update`.
#[derive(Debug, Clone)]
enum Message {
    TabSelected(TabId),
    NavigateTabForward,
    NavigateTabBackward,
    Pomodoro(pomodoro::Message),
    Settings(setting::Message),
    Report(report::Message),
}

/// Identifier for each application tab.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
enum TabId {
    Pomodoro,
    Setting,
    Report,
}

impl Default for App {
    fn default() -> Self {
        let settings = Settings::new();
        let pomodoro = Pomodoro::new(
            settings.work_min,
            settings.break_min,
            settings.work_theme.to_iced_theme(),
            settings.break_theme.to_iced_theme(),
        );

        Self {
            pomodoro,
            settings,
            active_tab: TabId::Pomodoro,
            report: Report::new(),
        }
    }
}

impl App {
    /// Processes messages to update the application state.
    ///
    /// This function acts as a central dispatcher, handling top-level messages and
    /// delegating component-specific messages to their respective update functions.
    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::TabSelected(id) => self.active_tab = id,
            Message::NavigateTabForward => {
                self.active_tab = match self.active_tab {
                    TabId::Pomodoro => TabId::Setting,
                    TabId::Setting => TabId::Report,
                    TabId::Report => TabId::Pomodoro,
                };
            }
            Message::NavigateTabBackward => {
                self.active_tab = match self.active_tab {
                    TabId::Pomodoro => TabId::Report,
                    TabId::Setting => TabId::Pomodoro,
                    TabId::Report => TabId::Setting,
                };
            }
            Message::Pomodoro(p_msg) => {
                // When a pomodoro day ends, generate a report and switch to the report tab.
                if matches!(p_msg, pomodoro::Message::EndDay) {
                    let (focused, completed) = self.pomodoro.get_completed_stats();
                    if completed > 0 {
                        self.report
                            .update(report::Message::Generate { focused, completed });
                        self.active_tab = TabId::Report;
                    }
                }

                return self.pomodoro.update(p_msg).map(Message::Pomodoro);
            }
            Message::Settings(s_msg) => {
                // When settings are submitted, apply them and switch to pomodoro tab.
                if matches!(s_msg, setting::Message::Submit) {
                    self.pomodoro.apply_settings(
                        self.settings.work_min,
                        self.settings.break_min,
                        self.settings.work_theme.to_iced_theme(),
                        self.settings.break_theme.to_iced_theme(),
                    );
                    self.active_tab = TabId::Pomodoro;
                }
                self.settings.update(s_msg);
            }
            Message::Report(r_msg) => self.report.update(r_msg),
        }

        iced::Task::none()
    }

    /// Defines application-wide subscriptions for timers and keyboard events.
    fn subscription(&self) -> Subscription<Message> {
        let pomodoro_sub = self.pomodoro.subscription().map(Message::Pomodoro);

        let tab_sub = iced::keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            Key::Named(Named::Tab) if modifiers.shift() => Some(Message::NavigateTabBackward),
            Key::Named(Named::Tab) => Some(Message::NavigateTabForward),
            _ => None,
        });

        Subscription::batch(vec![pomodoro_sub, tab_sub])
    }

    /// Constructs the user interface from the current application state.
    fn view(&self) -> Element<'_, Message> {
        let title = text("Icemodoro").size(30);
        let handler = image::Handle::from_bytes(include_bytes!("../logo.png").as_slice());
        let logo = image(handler).width(50).height(50);
        let header = row![title, text("").width(Length::Fill), logo].padding(10);

        let tabs = Tabs::new(Message::TabSelected)
            .push(
                TabId::Pomodoro,
                TabLabel::Text(self.pomodoro.tab_title()),
                self.pomodoro.view().map(Message::Pomodoro),
            )
            .push(
                TabId::Setting,
                TabLabel::Text("Setting".to_owned()),
                self.settings.view().map(Message::Settings),
            )
            .push(
                TabId::Report,
                TabLabel::Text("Report".to_owned()),
                self.report.view().map(Message::Report),
            )
            .set_active_tab(&self.active_tab)
            .tab_label_spacing(10);

        container(column![header, tabs].width(450))
            .center_x(Length::Fill)
            .padding(10)
            .into()
    }

    /// Provides the current theme, which changes dynamically based on the pomodoro session.
    fn theme(&self) -> Theme {
        self.pomodoro.theme()
    }
}
