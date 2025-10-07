//! Manages the report generating, storing, and viewing productivity reports, including streaks and focused time.

use super::persistence;

use iced::time::Duration;
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text};
use iced::{Center, Element, Length};

use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

/// Represents the productivity data collected for a single day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayReport {
    date: NaiveDate,
    focused: Duration,
    completed: usize,
}

/// Stores the complete Pomodoro usage history and summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    history: Vec<DayReport>,
    longest_streak: usize,
    longest_focused: Duration,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            longest_streak: 0,
            longest_focused: Duration::from_secs(0),
        }
    }
}

/// Messages used for updating the report tab.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Generate { focused: Duration, completed: usize },
    Clear,
    Import,
    Export,
}

impl Report {
    /// Loads the report state from persistent storage or returns default.
    pub fn new() -> Self {
        persistence::load("reports.json").unwrap_or_default()
    }

    /// Processes messages and updates the component's state.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Generate { completed, focused } => {
                let today = chrono::Local::now().date_naive();

                let current_focused = match self.history.iter_mut().find(|r| r.date == today) {
                    // Found today's report, update focused/completed values.
                    Some(report) => {
                        report.focused += focused;
                        report.completed += completed;
                        report.focused
                    }
                    // New day, add new report.
                    None => {
                        self.history.push(DayReport {
                            date: today,
                            focused,
                            completed,
                        });
                        self.history.sort_by_key(|r| r.date);

                        // Update longest streak if current streak is longer
                        let streak = self.calculate_current_streak(today);
                        if streak > self.longest_streak {
                            self.longest_streak = streak;
                        }

                        focused
                    }
                };

                if current_focused > self.longest_focused {
                    self.longest_focused = current_focused;
                }

                persistence::save("reports.json", &self).ok();
            }
            Message::Clear => {
                self.history.clear();
                self.longest_streak = 0;
                self.longest_focused = Duration::from_secs(0);
                persistence::save("reports.json", &self).ok();
            }
            Message::Import => {
                // TODO: Show error message on the view
                if let Ok(imported_data) = persistence::import::<Self>("reports.json") {
                    *self = imported_data;
                }
            }
            Message::Export => {
                // TODO: Show error message on the view
                persistence::export(&self).ok();
            }
        }
    }

    /// Calculates the current number of consecutive days with a report, ending with `today`.
    fn calculate_current_streak(&self, today: NaiveDate) -> usize {
        let history = &self.history;
        let mut day_streak = 0;
        let mut expected_date = Some(today);

        // Iterate backward through the history to check for continuity.
        for report in history.iter().rev() {
            match expected_date {
                Some(date) if report.date == date => {
                    day_streak += 1;
                    expected_date = date.checked_sub_days(Days::new(1));
                }
                _ => break,
            }
        }

        day_streak
    }

    /// Builds the report summary, history table, and control buttons.
    pub fn view(&self) -> Element<'_, Message> {
        let content = if self.history.is_empty() {
            column![
                text("No reports generated yet.").size(20),
                text("Press 'End Day' in the Pomodoro tab to save a report.").size(16),
                button("Import").on_press(Message::Import),
            ]
            .spacing(10)
        } else {
            let today = chrono::Local::now().date_naive();
            let day_streak = self.calculate_current_streak(today);
            let focused_today = match self.history.iter().find(|r| r.date == today) {
                Some(r) => r.focused,
                None => Duration::from_secs(0),
            };

            let summary_section = column![
                row![
                    column![
                        text("Current Day Streak:").size(18),
                        text("Longest Day Streak:").size(18),
                    ]
                    .width(Length::Fill),
                    column![
                        text(format!("{} days", day_streak)).size(18),
                        text(format!("{} days", self.longest_streak)).size(18),
                    ]
                    .width(Length::Fill),
                ]
                .spacing(10),
                row![
                    column![
                        text("Total Focused Today:").size(18),
                        text("Longest Focused Day:").size(18),
                    ]
                    .width(Length::Fill),
                    column![
                        text(format_duration(focused_today)).size(18),
                        text(format_duration(self.longest_focused)).size(18),
                    ]
                    .width(Length::Fill),
                ]
                .spacing(10)
            ]
            .spacing(10)
            .width(Length::Fill);

            let table_header = row![
                text("Date").width(Length::Fill),
                text("Focused Time").width(Length::Fill),
                text("Pomodoros").width(Length::Fill).align_x(Center),
            ]
            .spacing(10);

            // Generate report rows from history, showing most recent first.
            let report_rows: Vec<Element<_>> = self
                .history
                .iter()
                .rev()
                .map(|report| {
                    row![
                        text(report.date.format("%Y-%m-%d").to_string()).width(Length::Fill),
                        text(format_duration(report.focused)).width(Length::Fill),
                        text(report.completed.to_string())
                            .width(Length::Fill)
                            .align_x(Center),
                    ]
                    .spacing(10)
                    .padding(5)
                    .into()
                })
                .collect();

            let reports_list = column(report_rows).spacing(5);
            let history_buttons = container(
                row![
                    button("Export").on_press(Message::Export),
                    button("Clear")
                        .on_press(Message::Clear)
                        .style(button::danger)
                ]
                .spacing(20),
            )
            .center_x(Length::Fill);

            column![
                text("Summary").size(24),
                horizontal_rule(1),
                summary_section,
                text("History").size(24),
                horizontal_rule(1),
                table_header,
                horizontal_rule(1),
                scrollable(reports_list),
                history_buttons
            ]
            .spacing(10)
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(15)
            .into()
    }
}

/// Formats a `Duration` into an `HH:MM:SS` string.
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}")
}
