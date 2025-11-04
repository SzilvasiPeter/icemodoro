//! Manages the core Pomodoro timer, session state, and task list functionality.

use super::persistence;

use iced::keyboard::key::{Key, Named};
use iced::time::{Duration, Instant};
use iced::widget::text_input::Id;
use iced::widget::{
    button, column, container, horizontal_rule, keyed_column, progress_bar, row, text, text_input,
};
use iced::{Center, Element, Length, Subscription, Theme};

use notify_rust::Notification;
use rand::Rng;
use serde::{Deserialize, Serialize};

const BREAK_SUMMARIES: [&str; 7] = [
    "Stretch up high, touch the sky. Take a sip, stay fresh and spry.",
    "Bend and sway, greet the day. Drink your water, wash fatigue away.",
    "Twist with grace, find your space. Sip some water, keep your pace.",
    "Reach and glide, open wide. Hydrate well, feel joy inside.",
    "Roll your neck, take a sec. Drink your water, keep your check.",
    "Stretch with cheer, far and near. Take a sip, refresh your gear.",
    "Wiggle free, breathe with glee. Sip your water, let it be.",
];

const WORK_SUMMARIES: [&str; 7] = [
    "Shake off breaks and take your seat, make your tasks feel light and neat.",
    "Stretch was sweet, now tap your keys, move with calm and gentle ease.",
    "Sip was done, now face the day, let your work flow in a playful way.",
    "Shake the rest from head to toe, dive in now and let ideas grow.",
    "Mind refreshed, body bright, tackle tasks with all your might.",
    "Take a breath, then start the grind, joy and focus you will find.",
    "Break is gone, energy’s prime, back to work, it’s task time!",
];

/// Represents a single task in the to-do list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    id: u64,
    desc: String,
    spent: Duration,
    done: bool,
}

impl Task {
    fn new(id: u64, desc: String) -> Self {
        Self {
            id,
            desc,
            spent: Duration::ZERO,
            done: false,
        }
    }
}

/// Holds the state for the main Pomodoro timer and task management view.
pub struct Pomodoro {
    //-- Settings --//
    /// Duration of a work session.
    work_dur: Duration,
    /// The current number of finished work session.
    work_count: u8,
    /// Duration of a break session.
    break_dur: Duration,
    /// Duration of long break session.
    long_break_dur: Duration,
    /// Number of work session to start long break.
    long_break_after: u8,
    /// Theme used during work sessions.
    work_theme: Theme,
    /// Theme used during break sessions.
    break_theme: Theme,

    //-- Timer State --//
    /// The currently active theme.
    theme: Theme,
    /// The time left in the current session.
    remaining: Duration,
    /// The duration of the last completed session segment.
    last_done: Duration,
    /// Time elapsed after the timer reaches zero.
    overtime: Duration,
    /// The timer's operational state (e.g., Idle, Ticking).
    state: State,
    /// The current session type (Pomodoro or Break).
    session: Session,

    //-- Task State --//
    /// The list of all tasks.
    tasks: Vec<Task>,
    /// The ID to be assigned to the next new task.
    next_id: u64,
    /// The ID of the currently active task, if any.
    active: Option<u64>,
    /// The state of the task currently being edited `(id, description)`.
    editing: Option<(u64, String)>,
    /// A unique ID for the task editing input field.
    edit_id: Id,
    /// The current value of the new task input field.
    input: String,
    /// A unique ID for the new task input field.
    input_id: Id,
}

/// Messages used for updating the Pomodoro tab.
#[derive(Debug, Clone)]
pub enum Message {
    // Timer messages
    Tick(Instant),
    Toggle,
    Reset,
    Finish,

    // Task list messages
    Input(String),
    Add,
    Clear,
    EndDay,

    // Individual task messages
    Select(u64),
    Complete(u64),
    Delete(u64),
    Edit(u64),
    EditInput(String),
    SaveEdit,
    CancelEdit,

    // Keyboard shortcut messages
    FocusInput,
    CompleteActive,
    Activate,
    ActiveUp,
    ActiveDown,
    EditActive,
    DeleteActive,
}

/// Represents the operational state of the timer.
enum State {
    Idle,
    Ticking { expires: Instant },
    Overtime { last_tick: Instant },
}

/// Represents the type of session currently active.
#[derive(Debug)]
enum Session {
    Pomodoro,
    Break,
    LongBreak,
}

/// Used to indicate direction for moving the active task selection.
enum Direction {
    Up,
    Down,
}

impl Pomodoro {
    /// Initializes a new `Pomodoro` state with configured durations and themes.
    ///
    /// It also loads any existing tasks from persistent storage.
    pub fn new(
        work_min: u8,
        break_min: u8,
        long_break_min: u8,
        long_break_after: u8,
        work_theme: Theme,
        break_theme: Theme,
    ) -> Self {
        let work_duration = Duration::from_secs(u64::from(work_min) * 60);
        let break_duration = Duration::from_secs(u64::from(break_min) * 60);
        let long_break_duration = Duration::from_secs(u64::from(long_break_min) * 60);
        let tasks: Vec<Task> = persistence::load("tasks.json").unwrap_or_default();
        let active = tasks.iter().find(|t| !t.done).map(|t| t.id);

        // The initial task receives ID 1, and subsequent IDs increment from there.
        let next_id = tasks.iter().max_by_key(|t| t.id).map_or(1, |t| t.id + 1);

        Self {
            work_dur: work_duration,
            work_count: 0,
            break_dur: break_duration,
            long_break_dur: long_break_duration,
            long_break_after,
            work_theme: work_theme.clone(),
            break_theme,
            remaining: work_duration,
            overtime: Duration::ZERO,
            last_done: work_duration,
            state: State::Idle,
            session: Session::Pomodoro,
            theme: work_theme,
            tasks,
            active,
            next_id,
            editing: None,
            edit_id: Id::unique(),
            input: String::new(),
            input_id: Id::unique(),
        }
    }

    /// Updates the component's configuration from the settings.
    pub fn apply_settings(&mut self, w_min: u8, b_min: u8, w_theme: Theme, b_theme: Theme) {
        self.work_dur = Duration::from_secs(u64::from(w_min) * 60);
        self.break_dur = Duration::from_secs(u64::from(b_min) * 60);
        (self.work_theme, self.break_theme) = (w_theme, b_theme);

        // Only reset the timer if it's not currently running.
        if matches!(self.state, State::Idle) {
            self.reset_duration();
            self.theme = match self.session {
                Session::Pomodoro => self.work_theme.clone(),
                Session::Break | Session::LongBreak => self.break_theme.clone(),
            };
        }
    }

    /// Returns the count of completed tasks and the total time spent on them.
    pub fn get_completed_stats(&self) -> (Duration, usize) {
        let done_tasks: Vec<&Task> = self.tasks.iter().filter(|t| t.done).collect();
        let completed = done_tasks.len();
        let focused = done_tasks.iter().map(|t| t.spent).sum();
        (focused, completed)
    }

    /// Processes messages and updates the component's state.
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        // Any message that modifies the task list should trigger a save to disk.
        let task_updated = matches!(
            message,
            Message::Add
                | Message::Clear
                | Message::Finish
                | Message::Complete(_)
                | Message::CompleteActive
                | Message::SaveEdit
                | Message::Delete(_)
                | Message::DeleteActive
                | Message::EndDay
        );

        match message {
            // Timer messages
            Message::Tick(now) => self.handle_tick(now),
            Message::Toggle => self.toogle_timer(),
            Message::Reset => self.reset_duration(),
            Message::Finish => self.finish_timer(),

            // Task list messages
            Message::Input(value) => self.input = value,
            Message::Add => {
                let desc = self.input.trim().to_string();
                if !desc.is_empty() {
                    self.tasks.push(Task::new(self.next_id, desc));
                    self.next_id = self.next_id.wrapping_add(1);
                    self.input.clear();
                }
            }
            Message::Clear => self.tasks.clear(),
            Message::EndDay => self.tasks.retain(|task| !task.done),

            // Individual task messages
            Message::Select(id) => self.select_task(id),
            Message::Complete(id) => self.complete_task(id),
            Message::Delete(id) => self.delete_task(id),
            Message::Edit(id) => self.edit_task(id),
            Message::EditInput(input) => {
                if let Some(editing) = &mut self.editing {
                    editing.1 = input;
                }
            }
            Message::SaveEdit => {
                if let Some((id, new_text)) = self.editing.take()
                    && !new_text.trim().is_empty()
                    && let Some(task) = self.tasks.iter_mut().find(|t| t.id == id)
                {
                    task.desc = new_text;
                }
            }
            Message::CancelEdit => self.editing = None,

            // Keyboard shortcut messages
            Message::FocusInput => return text_input::focus(self.input_id.clone()),
            Message::CompleteActive => {
                if let Some(active_id) = self.active {
                    self.complete_task(active_id);
                }
            }
            Message::Activate => {
                if let Some(task) = self.tasks.iter().find(|task| !task.done) {
                    self.select_task(task.id);
                }
            }
            Message::ActiveUp => self.move_active(&Direction::Up),
            Message::ActiveDown => self.move_active(&Direction::Down),
            Message::EditActive => {
                if let Some(active_id) = self.active {
                    self.edit_task(active_id);
                }
                return text_input::focus(self.edit_id.clone());
            }
            Message::DeleteActive => {
                if let Some(active_id) = self.active {
                    self.delete_task(active_id);
                }
            }
        }

        if task_updated {
            persistence::save("tasks.json", &self.tasks).ok();
        }

        iced::Task::none()
    }

    /// Defines subscriptions for timer ticks and keyboard shortcuts.
    pub fn subscription(&self) -> Subscription<Message> {
        let timer_sub = match self.state {
            State::Idle => Subscription::none(),
            _ => iced::time::every(Duration::from_millis(500)).map(Message::Tick),
        };

        let key_sub = iced::keyboard::on_key_press(|key, _modifiers| match key.as_ref() {
            Key::Named(Named::Space) => Some(Message::Toggle),
            Key::Character("r") => Some(Message::Reset),
            Key::Character("f") => Some(Message::Finish),
            Key::Character("n") => Some(Message::FocusInput),
            Key::Character("s") => Some(Message::CompleteActive),
            Key::Character("a") => Some(Message::Activate),
            Key::Named(Named::ArrowUp) => Some(Message::ActiveUp),
            Key::Named(Named::ArrowDown) => Some(Message::ActiveDown),
            Key::Character("e") => Some(Message::EditActive),
            Key::Character("d") => Some(Message::DeleteActive),
            Key::Character("x") => Some(Message::EndDay),
            _ => None,
        });

        Subscription::batch(vec![timer_sub, key_sub])
    }

    /// Builds the main view for the Pomodoro tab.
    pub fn view(&self) -> Element<'_, Message> {
        let max_range = match self.session {
            Session::Pomodoro => self.work_dur,
            Session::Break => self.break_dur,
            Session::LongBreak => self.long_break_dur,
        };
        let progress = progress_bar(0.0..=max_range.as_secs_f32(), self.remaining.as_secs_f32());
        column![progress.height(1), self.view_timer(), self.view_tasks()]
            .align_x(Center)
            .padding(10)
            .into()
    }

    /// Returns the title for the Pomodoro tab, indicating the current session.
    pub fn tab_title(&self) -> String {
        match self.session {
            Session::LongBreak => "Long Break".to_string(),
            Session::Pomodoro | Session::Break => format!("{:?}", self.session),
        }
    }

    /// Returns the currently active theme.
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// Counts down the timer. If timer is zero, then notifies and counts up the overtime.
    fn handle_tick(&mut self, now: Instant) {
        match &mut self.state {
            State::Ticking { expires } => {
                if let Some(duration) = expires.checked_duration_since(now) {
                    self.remaining = duration;
                } else {
                    match self.session {
                        Session::Break | Session::LongBreak => {
                            let index = rand::rng().random_range(0..WORK_SUMMARIES.len());
                            let _ = Notification::new()
                                .sound_name("alarm-clock-elapsed")
                                .summary(WORK_SUMMARIES[index])
                                .show();
                        }
                        Session::Pomodoro => {
                            let index = rand::rng().random_range(0..BREAK_SUMMARIES.len());
                            let _ = Notification::new()
                                .sound_name("alarm-clock-elapsed")
                                .summary(BREAK_SUMMARIES[index])
                                .show();
                        }
                    }
                    self.remaining = Duration::ZERO;
                    self.state = State::Overtime { last_tick: now };
                }
            }
            State::Overtime { last_tick } => {
                self.overtime = self.overtime.saturating_add(now - *last_tick);
                *last_tick = now;
            }
            State::Idle => {}
        }
    }

    /// Starts or stops the timer.
    fn toogle_timer(&mut self) {
        let expires = Instant::now() + self.remaining;
        self.state = match self.state {
            State::Idle => State::Ticking { expires },
            _ => State::Idle,
        }
    }

    /// Logs the work time spent on the active task, then switches to the next session type.
    fn finish_timer(&mut self) {
        if let (Session::Pomodoro, Some(id)) = (&self.session, self.active) {
            let time_spent = self.get_time_spent();
            if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
                task.spent = task.spent.saturating_add(time_spent);
            }
            self.work_count += 1;
        }

        self.session = match self.session {
            Session::Pomodoro if self.work_count >= self.long_break_after => {
                self.work_count = 0;
                self.theme = self.break_theme.clone();
                Session::LongBreak
            }
            Session::Pomodoro => {
                self.theme = self.break_theme.clone();
                Session::Break
            }
            Session::Break | Session::LongBreak => {
                self.theme = self.work_theme.clone();
                Session::Pomodoro
            }
        };
        self.reset_duration();
    }

    /// Resets the timer to the current session's full duration.
    fn reset_duration(&mut self) {
        self.remaining = match self.session {
            Session::Pomodoro => self.work_dur,
            Session::Break => self.break_dur,
            Session::LongBreak => self.long_break_dur,
        };
        self.last_done = self.remaining;
        self.overtime = Duration::ZERO;
        self.state = State::Idle;
    }

    /// Calculates the total time spent in the current pomodoro segment.
    fn get_time_spent(&self) -> Duration {
        if self.remaining.is_zero() {
            self.last_done + self.overtime
        } else {
            self.last_done - self.remaining
        }
    }

    /// Selects or deselects a task as active.
    fn select_task(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.done = false;
        }
        self.active = (self.active != Some(id)).then_some(id);
        self.state = State::Idle;
    }

    /// Moves the active task selection up or down from the list of incomplete tasks.
    fn move_active(&mut self, direction: &Direction) {
        let active_tasks: Vec<&Task> = self.tasks.iter().filter(|task| !task.done).collect();
        let Some(current_active_id) = self.active else {
            return;
        };

        let current_index = active_tasks
            .iter()
            .position(|task| task.id == current_active_id)
            .unwrap_or(0);

        // Activate the next or previous task, wrapping around at the ends.
        let new_index = match direction {
            Direction::Up => (current_index + active_tasks.len() - 1) % active_tasks.len(),
            Direction::Down => (current_index + 1) % active_tasks.len(),
        };

        self.active = active_tasks.get(new_index).map(|task| task.id);
        self.state = State::Idle;
    }

    /// Toggles the completion status of a task and logs the time spent.
    fn complete_task(&mut self, id: u64) {
        let time_spent = self.get_time_spent();
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.done = !task.done;
            task.spent = task.spent.saturating_add(time_spent);
        }

        self.last_done = self.remaining;
        self.overtime = Duration::ZERO;
        self.state = State::Idle;
        self.active = self
            .tasks
            .iter()
            .find(|task| !task.done)
            .map(|task| task.id);
    }

    /// Puts a task into editing mode.
    fn edit_task(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter().find(|task| task.id == id) {
            self.editing = Some((task.id, task.desc.clone()));
        }
    }

    /// Deletes a task from the list.
    fn delete_task(&mut self, id: u64) {
        self.tasks.retain(|task| task.id != id);
        if let Some(task) = self.tasks.iter().find(|task| !task.done) {
            self.active = Some(task.id);
        }
    }

    /// View section for the timer display and controls.
    fn view_timer(&self) -> Element<'_, Message> {
        let remaining_secs = self.remaining.as_secs();
        let duration_text = format!("{}:{:0>2}", remaining_secs / 60, remaining_secs % 60);

        let overtime_secs = self.overtime.as_secs();
        let overtime_widget = if let State::Overtime { .. } = self.state {
            column![text!("+{}:{:0>2}", overtime_secs / 60, overtime_secs % 60).size(16)]
        } else {
            column![]
        };

        let is_idle = matches!(self.state, State::Idle);
        let toggle_text = if is_idle { "Start" } else { "Pause" };

        column![
            text(duration_text).size(40),
            overtime_widget,
            row![
                button(toggle_text).on_press(Message::Toggle),
                button("Reset").on_press(Message::Reset),
                button("Finish").on_press(Message::Finish),
            ]
            .spacing(20),
        ]
        .spacing(10)
        .align_x(Center)
        .into()
    }

    /// View section for the task list and input form.
    fn view_tasks(&self) -> Element<'_, Message> {
        let tasks_list = self.tasks.iter().map(|task| {
            let view: Element<_> = match self.editing.as_ref() {
                // Render the editing view for the selected task.
                Some((id, desc)) if *id == task.id => row![
                    text_input("Edit task...", desc)
                        .id(self.edit_id.clone())
                        .on_input(Message::EditInput)
                        .on_submit(Message::SaveEdit),
                    button("Save").on_press(Message::SaveEdit),
                    button("Cancel").on_press(Message::CancelEdit),
                ]
                .spacing(10)
                .align_y(Center)
                .into(),
                // Render the normal view for all other tasks.
                _ => {
                    let done_icon = if task.done {
                        text("⊗").shaping(text::Shaping::Advanced)
                    } else {
                        text("⊙").shaping(text::Shaping::Advanced)
                    };
                    let task_style = match (self.active == Some(task.id), task.done) {
                        (true, _) => button::primary,
                        (false, true) => button::success,
                        _ => button::secondary,
                    };

                    row![
                        button(done_icon)
                            .on_press(Message::Complete(task.id))
                            .style(task_style),
                        button(text(&task.desc))
                            .style(task_style)
                            .width(Length::Fill)
                            .on_press(Message::Select(task.id)),
                        text(format_duration(task.spent)),
                        button(text("⋯").shaping(text::Shaping::Advanced))
                            .on_press(Message::Edit(task.id)),
                        button(text("×").shaping(text::Shaping::Advanced))
                            .style(button::danger)
                            .on_press(Message::Delete(task.id)),
                    ]
                    .spacing(10)
                    .align_y(Center)
                    .into()
                }
            };
            (task.id, view)
        });

        let add_fields = if self.tasks.len() <= 10 {
            row![
                text_input("What are you working on?", &self.input)
                    .id(self.input_id.clone())
                    .on_input(Message::Input)
                    .on_submit(Message::Add),
                button("Add").on_press(Message::Add)
            ]
        } else {
            row![
                text_input("Let’s finish current tasks first!", &self.input),
                button("Add")
            ]
        };

        column![
            text("Tasks").size(20),
            horizontal_rule(1),
            row![
                add_fields.spacing(10),
                button("Delete All")
                    .on_press(Message::Clear)
                    .style(button::danger)
            ]
            .spacing(10),
            keyed_column(tasks_list).spacing(10),
            container(
                button("End Day")
                    .on_press(Message::EndDay)
                    .style(button::success)
            )
            .center_x(Length::Fill),
        ]
        .spacing(20)
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
