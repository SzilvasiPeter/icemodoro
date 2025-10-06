use super::persistence;

use iced::keyboard::key::{Key, Named};
use iced::time::{Duration, Instant, every};
use iced::widget::text_input::Id;
use iced::widget::{
    button, column, container, horizontal_rule, keyed_column, progress_bar, row, text, text_input,
};
use iced::{Center, Element, Length, Subscription, Theme};

use notify_rust::Notification;
use serde::{Deserialize, Serialize};

// TODO: Add documentation
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

pub struct Pomodoro {
    work_dur: Duration,
    break_dur: Duration,
    work_theme: Theme,
    break_theme: Theme,
    theme: Theme,
    remaining: Duration,
    last_done: Duration,
    overtime: Duration,
    state: State,
    session: Session,
    tasks: Vec<Task>,
    next_id: u64,
    active: Option<u64>,
    editing: Option<(u64, String)>,
    edit_id: Id,
    input: String,
    input_id: Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Timer specific messages
    Tick(Instant),
    Toggle,
    Reset,
    Finish,

    // General task messages
    Input(String),
    Add,
    Clear,
    EndDay,

    // Task item messages
    Select(u64),
    Complete(u64),
    Delete(u64),
    Edit(u64),
    EditInput(String),
    SaveEdit,
    CancelEdit,

    // Shortcut specific messages
    FocusInput,
    CompleteActive,
    Activate,
    ActiveUp,
    ActiveDown,
    EditActive,
    DeleteActive,
}

enum State {
    Idle,
    Ticking { expires: Instant },
    Overtime { last_tick: Instant },
}

#[derive(Debug)]
enum Session {
    Pomodoro,
    Break,
}

enum Direction {
    Up,
    Down,
}

impl Pomodoro {
    pub fn new(work_min: u8, break_min: u8, work_theme: Theme, break_theme: Theme) -> Self {
        let work_duration = Duration::from_secs(work_min as u64 * 60);
        let break_duration = Duration::from_secs(break_min as u64 * 60);
        let tasks: Vec<Task> = persistence::load("tasks.json").unwrap_or_default();
        let active = tasks.iter().find(|t| !t.done).and_then(|t| Some(t.id));
        let next_id = tasks.iter().max_by_key(|t| t.id).map_or(1, |t| t.id + 1);
        Self {
            work_dur: work_duration,
            break_dur: break_duration,
            work_theme: work_theme.clone(),
            break_theme: break_theme,
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

    pub fn apply_settings(&mut self, w_min: u8, b_min: u8, w_theme: Theme, b_theme: Theme) {
        self.work_dur = Duration::from_secs(w_min as u64 * 60);
        self.break_dur = Duration::from_secs(b_min as u64 * 60);
        (self.work_theme, self.break_theme) = (w_theme, b_theme);

        if let State::Idle = self.state {
            self.reset_duration();
            self.theme = match self.session {
                Session::Pomodoro => self.work_theme.clone(),
                Session::Break => self.break_theme.clone(),
            };
        }
    }

    pub fn tab_title(&self) -> String {
        format!("{:?}", self.session)
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn get_completed_stats(&self) -> (usize, Duration) {
        let done_tasks: Vec<Task> = self.tasks.iter().filter(|t| t.done).cloned().collect();
        let task_count = done_tasks.len();
        let total_time = done_tasks.iter().map(|t| t.spent).sum();
        (task_count, total_time)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let timer_sub = match self.state {
            State::Idle => Subscription::none(),
            _ => every(Duration::from_millis(500)).map(Message::Tick),
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

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        let task_updated = matches!(
            message,
            Message::Add
                | Message::Clear
                | Message::Complete(_)
                | Message::CompleteActive
                | Message::SaveEdit
                | Message::Delete(_)
                | Message::DeleteActive
                | Message::EndDay
        );

        match message {
            Message::Tick(now) => match &mut self.state {
                State::Ticking { expires } => {
                    if let Some(duration) = expires.checked_duration_since(now) {
                        self.remaining = duration;
                    } else {
                        self.remaining = Duration::ZERO;
                        Notification::new()
                            .sound_name("alarm-clock-elapsed")
                            .summary("Overtime!")
                            .show()
                            .ok();
                        self.state = State::Overtime { last_tick: now };
                    }
                }
                State::Overtime { last_tick } => {
                    self.overtime += now - *last_tick;
                    *last_tick = now;
                }
                State::Idle => {}
            },
            Message::Toggle => {
                let expires = Instant::now() + self.remaining;
                self.state = match self.state {
                    State::Idle => State::Ticking { expires },
                    _ => State::Idle,
                }
            }
            Message::Reset => self.reset_duration(),
            Message::Finish => {
                if let (Session::Pomodoro, Some(id)) = (&self.session, self.active) {
                    let time_spent = self.get_time_spent();
                    if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
                        task.spent += time_spent;
                    }
                }

                self.session = match self.session {
                    Session::Pomodoro => {
                        self.theme = self.break_theme.clone();
                        Session::Break
                    }
                    Session::Break => {
                        self.theme = self.work_theme.clone();
                        Session::Pomodoro
                    }
                };
                self.reset_duration();
            }
            Message::Input(value) => self.input = value,
            Message::Add => {
                let desc = self.input.trim().to_string();
                if !desc.is_empty() {
                    self.tasks.push(Task::new(self.next_id, desc));
                    self.next_id += 1;
                    self.input.clear();
                }
            }
            Message::Clear => self.tasks.clear(),
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
            Message::EndDay => self.tasks.retain(|task| !task.done),
            Message::FocusInput => return text_input::focus(self.input_id.clone()),
            Message::CompleteActive => self.complete_task(self.active.unwrap_or(0)),
            Message::Activate => {
                if let Some(task) = self.tasks.iter().find(|task| !task.done) {
                    self.select_task(task.id);
                }
            }
            Message::ActiveUp => self.move_active(Direction::Up),
            Message::ActiveDown => self.move_active(Direction::Down),
            Message::EditActive => {
                self.edit_task(self.active.unwrap_or(0));
                return text_input::focus(self.edit_id.clone());
            }
            Message::DeleteActive => self.delete_task(self.active.unwrap_or(0)),
        }

        if task_updated {
            persistence::save("tasks.json", &self.tasks).ok();
        }

        // We will return a `text_input::focus` actions
        // when adding or editing task to avoid mouse interaction.
        // In any other case, no iced action is necessary.
        iced::Task::none()
    }

    fn get_time_spent(&self) -> Duration {
        if self.remaining.is_zero() {
            self.last_done + self.overtime
        } else {
            self.last_done - self.remaining
        }
    }

    fn complete_task(&mut self, id: u64) {
        let time_spent = self.get_time_spent();
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.done = !task.done;
            task.spent += time_spent;
        }

        self.last_done = self.remaining;
        self.overtime = Duration::ZERO;
        self.state = State::Idle;

        self.active = match self.tasks.iter().find(|task| !task.done) {
            Some(task) => Some(task.id),
            None => None,
        };
    }

    fn select_task(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.done = false;
        }
        self.active = (self.active != Some(id)).then_some(id);
        self.state = State::Idle;
    }

    fn move_active(&mut self, direction: Direction) {
        let active_tasks: Vec<&Task> = self.tasks.iter().filter(|task| !task.done).collect();
        let current_active_id = match self.active {
            Some(id) => id,
            None => return,
        };

        let current_index = active_tasks
            .iter()
            .position(|task| task.id == current_active_id)
            .unwrap_or(0);

        // Activate the next or the previous task, wrapping around at the ends.
        let new_index = match direction {
            Direction::Up => (current_index + active_tasks.len() - 1) % active_tasks.len(),
            Direction::Down => (current_index + 1) % active_tasks.len(),
        };

        self.active = active_tasks.get(new_index).map(|task| task.id);
        self.state = State::Idle;
    }

    fn delete_task(&mut self, id: u64) {
        self.tasks.retain(|task| task.id != id);
        if let Some(task) = self.tasks.iter().find(|task| !task.done) {
            self.active = Some(task.id);
        }
    }

    fn edit_task(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter().find(|task| task.id == id) {
            self.editing = Some((task.id, task.desc.clone()));
        }
    }

    fn reset_duration(&mut self) {
        self.remaining = match self.session {
            Session::Pomodoro => self.work_dur,
            Session::Break => self.break_dur,
        };
        self.last_done = self.remaining;
        self.overtime = Duration::ZERO;
        self.state = State::Idle;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let max_range = match self.session {
            Session::Pomodoro => self.work_dur,
            Session::Break => self.break_dur,
        };
        let progress = progress_bar(0.0..=max_range.as_secs_f32(), self.remaining.as_secs_f32());
        column![progress.height(1), self.view_timer(), self.view_tasks()]
            .align_x(Center)
            .padding(10)
            .into()
    }

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

    fn view_tasks(&self) -> Element<'_, Message> {
        let tasks_list = self.tasks.iter().map(|task| {
            let view: Element<_> = match self.editing.as_ref() {
                // Edit view
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
                // Normal view
                _ => {
                    let done_icon = if task.done { "⊗" } else { "⊙" };
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
                        button("⁝").on_press(Message::Edit(task.id)),
                        button("×")
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

        column![
            text("Tasks").size(20),
            horizontal_rule(1),
            row![
                text_input("What are you working on?", &self.input)
                    .id(self.input_id.clone())
                    .on_input(Message::Input)
                    .on_submit(Message::Add),
                button("Add").on_press(Message::Add),
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

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}")
}
