// File Rename Plus - Cross-platform file renaming application with native GUI

#![warn(clippy::all)]
#![windows_subsystem = "windows"]

mod app;
mod file_ops;
mod rename;
mod security;
mod settings;
mod theme;
mod types;

use app::FileRenamePlus;
use iced::{application, Font, Settings, Size};
use theme::{WINDOW_HEIGHT, WINDOW_WIDTH};

fn main() -> iced::Result {
    application(
        "File Rename Plus",
        FileRenamePlus::update,
        FileRenamePlus::view,
    )
    .theme(FileRenamePlus::theme)
    .subscription(FileRenamePlus::subscription)
    .settings(Settings {
        default_font: Font::DEFAULT,
        default_text_size: theme::FONT_MD.into(),
        antialiasing: true,
        ..Settings::default()
    })
    .window_size(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT))
    .run_with(FileRenamePlus::new)
}
