# Icemodoro
Pomodoro and ToDo application, inspired by [pomofocus.io](https://pomofocus.io/), using the Iced GUI library written in Rust.

![screenshot](screenshot.png "Icemodoro screenshot")

## Features

- Pomodoro timer with customizable durations and themes
- Tasks, settings, and reports are saved automatically
- Tracks daily focused time, completed sessions, and calculates streaks
- Import/export functionality for report

## Installation

Compile and run from the source:

```bash
cargo run --release
```

Or install the standalone binary:

```bash
cargo-binstall icemodoro
```

> Note: Download is available on 64-bit Linux (glibc), Intel or Apple Silicon Macs (via Rosetta), and 64-bit Windows PCs (GNU toolchain).

## Shortcuts

| Key | Action |
| :--- | :--- |
| **Space** | Start/Stop timer |
| **r** | Reset timer |
| **f** | Finish session (skip to next) |
| **n** | Focus new task input |
| **a** | Activate/Deactivate first task |
| **↑ / ↓** | Navigate active task |
| **s** | Complete active task |
| **e** | Edit active task |
| **d** | Delete active task |
| **x** | End day (generates daily report) |
| **Ctrl + Tab** | Next tab |
| **Shift + Tab** | Previous tab |
