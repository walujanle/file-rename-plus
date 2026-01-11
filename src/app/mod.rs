// Application state and iced GUI implementation

use crate::file_ops::{scan_directory, validate_and_rename};
use crate::rename::{apply_find_replace, apply_iteration_numbering};
use crate::security::can_modify_file;
use crate::settings::{load_settings, save_settings, Settings};
use crate::theme::{
    COLOR_CONFLICT, COLOR_ERROR, COLOR_INFO, COLOR_MUTED_DARK, COLOR_SUCCESS, FONT_LG, FONT_SM,
    FONT_XL, LIST_HEIGHT, MAX_FILES, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};
use crate::types::{AppMode, FileEntry, RenamePreview};
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, row,
    scrollable, text, text_input, vertical_space, Column,
};
use iced::{keyboard, time, Center, Element, Fill, Subscription, Task, Theme};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const DEBOUNCE_MS: u64 = 300;

pub struct FileRenamePlus {
    mode: AppMode,
    files: Vec<FileEntry>,
    selected_index: Option<usize>,
    find_pattern: String,
    replace_with: String,
    regex_mode: bool,
    case_sensitive: bool,
    template: String,
    start_number: String,
    padding: String,
    previews: Vec<RenamePreview>,
    status_message: Option<String>,
    is_error: bool,
    dark_mode: bool,
    last_input_time: Option<Instant>,
    pending_preview: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ModeChanged(AppMode),
    AddFolder,
    FolderSelected(Option<PathBuf>),
    ScanCompleted(Result<Vec<FileEntry>, String>),
    FileSelected(usize),
    MoveUp,
    MoveDown,
    RemoveFile,
    ClearFiles,
    FindPatternChanged(String),
    ReplaceWithChanged(String),
    RegexModeToggled(bool),
    CaseSensitiveToggled(bool),
    TemplateChanged(String),
    StartNumberChanged(String),
    PaddingChanged(String),
    ExecuteRename,
    RenameCompleted(Result<usize, String>),
    ToggleTheme,
    SettingsSaved,
    DebounceTick,
    KeyboardEvent(keyboard::Key, keyboard::Modifiers),
}

impl FileRenamePlus {
    // Creates new app instance, loads saved settings
    pub fn new() -> (Self, Task<Message>) {
        let settings = load_settings();
        (
            Self {
                mode: AppMode::FindReplace,
                files: Vec::new(),
                selected_index: None,
                find_pattern: String::new(),
                replace_with: String::new(),
                regex_mode: settings.regex_mode,
                case_sensitive: settings.case_sensitive,
                template: settings.template,
                start_number: settings.start_number.to_string(),
                padding: settings.padding.to_string(),
                previews: Vec::new(),
                status_message: Some("Click 'Add Folder' or press Ctrl+O".to_string()),
                is_error: false,
                dark_mode: settings.dark_mode,
                last_input_time: None,
                pending_preview: false,
            },
            Task::none(),
        )
    }

    // Creates Settings struct from current state
    fn to_settings(&self) -> Settings {
        Settings {
            dark_mode: self.dark_mode,
            regex_mode: self.regex_mode,
            case_sensitive: self.case_sensitive,
            template: self.template.clone(),
            start_number: self.start_number.parse().unwrap_or(1),
            padding: self.padding.parse().unwrap_or(3),
        }
    }

    // Saves settings asynchronously
    fn save_settings_async(&self) -> Task<Message> {
        let settings = self.to_settings();
        Task::perform(
            async move {
                save_settings(&settings);
            },
            |()| Message::SettingsSaved,
        )
    }

    // Schedules debounced preview generation
    fn schedule_preview(&mut self) {
        self.last_input_time = Some(Instant::now());
        self.pending_preview = true;
    }

    pub fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            {
                Some(Message::KeyboardEvent(key, modifiers))
            } else {
                None
            }
        });

        let debounce_sub = if self.pending_preview {
            time::every(Duration::from_millis(50)).map(|_| Message::DebounceTick)
        } else {
            Subscription::none()
        };

        Subscription::batch([keyboard_sub, debounce_sub])
    }

    // Handles all application messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyboardEvent(key, modifiers) => {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Delete) => {
                        return self.update(Message::RemoveFile);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter) if modifiers.control() => {
                        return self.update(Message::ExecuteRename);
                    }
                    keyboard::Key::Character(c) if modifiers.control() && c.as_str() == "o" => {
                        return self.update(Message::AddFolder);
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::DebounceTick => {
                if let Some(last_time) = self.last_input_time {
                    if last_time.elapsed() >= Duration::from_millis(DEBOUNCE_MS) {
                        self.pending_preview = false;
                        self.last_input_time = None;
                        self.generate_preview();
                    }
                }
                Task::none()
            }
            Message::SettingsSaved => Task::none(),
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                self.save_settings_async()
            }
            Message::ModeChanged(mode) => {
                self.mode = mode;
                self.generate_preview();
                Task::none()
            }
            Message::AddFolder => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Select Folder")
                        .pick_folder()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                Message::FolderSelected,
            ),
            Message::FolderSelected(path) => {
                if let Some(path) = path {
                    self.status_message = Some("Scanning...".to_string());
                    self.is_error = false;
                    let path_str = path.to_string_lossy().to_string();
                    Task::perform(
                        async move { scan_directory(&path_str).map_err(|e| e.to_string()) },
                        Message::ScanCompleted,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ScanCompleted(result) => {
                match result {
                    Ok(entries) => {
                        for entry in entries {
                            if self.files.len() >= MAX_FILES {
                                self.status_message = Some(format!("Max {} files", MAX_FILES));
                                break;
                            }
                            if !self.files.iter().any(|f| f.path == entry.path) {
                                self.files.push(entry);
                            }
                        }
                        self.status_message = Some(format!("Total: {} files", self.files.len()));
                        self.is_error = false;
                        self.generate_preview();
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error: {}", e));
                        self.is_error = true;
                    }
                }
                Task::none()
            }
            Message::FileSelected(index) => {
                self.selected_index = Some(index);
                Task::none()
            }
            Message::MoveUp => {
                if let Some(i) = self.selected_index {
                    if i > 0 {
                        self.files.swap(i, i - 1);
                        self.selected_index = Some(i - 1);
                        self.generate_preview();
                    }
                }
                Task::none()
            }
            Message::MoveDown => {
                if let Some(i) = self.selected_index {
                    if i < self.files.len().saturating_sub(1) {
                        self.files.swap(i, i + 1);
                        self.selected_index = Some(i + 1);
                        self.generate_preview();
                    }
                }
                Task::none()
            }
            Message::RemoveFile => {
                if let Some(i) = self.selected_index {
                    if i < self.files.len() {
                        self.files.remove(i);
                        self.selected_index = if self.files.is_empty() {
                            None
                        } else if i >= self.files.len() {
                            Some(self.files.len() - 1)
                        } else {
                            Some(i)
                        };
                        self.generate_preview();
                    }
                }
                Task::none()
            }
            Message::ClearFiles => {
                self.files.clear();
                self.selected_index = None;
                self.previews.clear();
                self.status_message = Some("All files cleared".to_string());
                self.is_error = false;
                Task::none()
            }
            Message::FindPatternChanged(p) => {
                self.find_pattern = p;
                self.schedule_preview();
                Task::none()
            }
            Message::ReplaceWithChanged(t) => {
                self.replace_with = t;
                self.schedule_preview();
                Task::none()
            }
            Message::RegexModeToggled(e) => {
                self.regex_mode = e;
                self.generate_preview();
                self.save_settings_async()
            }
            Message::CaseSensitiveToggled(e) => {
                self.case_sensitive = e;
                self.generate_preview();
                self.save_settings_async()
            }
            Message::TemplateChanged(t) => {
                self.template = t;
                self.schedule_preview();
                self.save_settings_async()
            }
            Message::StartNumberChanged(n) => {
                self.start_number = n;
                self.schedule_preview();
                self.save_settings_async()
            }
            Message::PaddingChanged(p) => {
                self.padding = p;
                self.schedule_preview();
                self.save_settings_async()
            }
            Message::ExecuteRename => {
                if self.previews.is_empty() {
                    self.status_message = Some("No changes to apply".to_string());
                    self.is_error = true;
                    return Task::none();
                }
                for preview in &self.previews {
                    if !can_modify_file(&preview.original_path) {
                        self.status_message = Some(format!(
                            "Access denied: {}",
                            preview.original_path.display()
                        ));
                        self.is_error = true;
                        return Task::none();
                    }
                }
                let previews = self.previews.clone();
                Task::perform(
                    async move { validate_and_rename(&previews).map_err(|e| e.to_string()) },
                    Message::RenameCompleted,
                )
            }
            Message::RenameCompleted(result) => {
                match result {
                    Ok(count) => {
                        self.status_message = Some(format!("Renamed {} file(s)!", count));
                        self.is_error = false;
                        self.files.clear();
                        self.previews.clear();
                        self.selected_index = None;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error: {}", e));
                        self.is_error = true;
                    }
                }
                Task::none()
            }
        }
    }

    // Generates rename preview based on current mode and settings
    fn generate_preview(&mut self) {
        self.previews.clear();
        if self.files.is_empty() {
            return;
        }

        match self.mode {
            AppMode::FindReplace => {
                if self.find_pattern.is_empty() {
                    self.status_message = Some("Enter a pattern to find".to_string());
                    return;
                }
                match apply_find_replace(
                    &self.files,
                    &self.find_pattern,
                    &self.replace_with,
                    self.regex_mode,
                    self.case_sensitive,
                ) {
                    Ok(p) => {
                        self.previews = p;
                        self.status_message = Some(if self.previews.is_empty() {
                            "No matches".to_string()
                        } else {
                            format!("{} file(s) matched", self.previews.len())
                        });
                        self.is_error = false;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error: {}", e));
                        self.is_error = true;
                    }
                }
            }
            AppMode::Iteration => {
                match apply_iteration_numbering(
                    &self.files,
                    &self.template,
                    self.start_number.parse().unwrap_or(1),
                    self.padding.parse().unwrap_or(3),
                ) {
                    Ok(p) => {
                        self.previews = p;
                        self.status_message =
                            Some(format!("{} file(s) ready", self.previews.len()));
                        self.is_error = false;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error: {}", e));
                        self.is_error = true;
                    }
                }
            }
        }
    }

    // Renders main application view
    pub fn view(&self) -> Element<'_, Message> {
        let content = row![self.view_file_list(), self.view_preview()].spacing(SPACING_MD);
        container(
            column![
                self.view_header(),
                vertical_space().height(SPACING_MD),
                content,
                vertical_space().height(SPACING_MD),
                self.view_options(),
                vertical_space().height(SPACING_MD),
                self.view_status(),
            ]
            .padding(SPACING_LG),
        )
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let theme_label = if self.dark_mode {
            "Light Mode"
        } else {
            "Dark Mode"
        };
        row![
            text("File Rename Plus").size(FONT_XL),
            horizontal_space(),
            button(theme_label).on_press(Message::ToggleTheme),
            text("  Mode: ").size(FONT_LG),
            pick_list(
                vec![AppMode::FindReplace, AppMode::Iteration],
                Some(self.mode),
                Message::ModeChanged
            )
            .width(200),
        ]
        .align_y(Center)
        .into()
    }

    fn view_file_list(&self) -> Element<'_, Message> {
        let header = row![
            text("Files").size(FONT_LG),
            horizontal_space(),
            button("Add Folder (Ctrl+O)").on_press(Message::AddFolder),
            button("Clear").on_press(Message::ClearFiles)
        ]
        .spacing(SPACING_SM)
        .align_y(Center);

        let file_buttons: Vec<Element<'_, Message>> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let btn = button(text(f.name.as_str()).size(FONT_SM))
                    .width(Fill)
                    .on_press(Message::FileSelected(i));
                if self.selected_index == Some(i) {
                    btn.style(button::primary).into()
                } else {
                    btn.style(button::secondary).into()
                }
            })
            .collect();

        let file_list = if file_buttons.is_empty() {
            column![text("No files. Click 'Add Folder'.").size(FONT_SM)]
        } else {
            Column::with_children(file_buttons).spacing(SPACING_XS)
        };

        let controls = row![
            button("Up").on_press(Message::MoveUp),
            button("Down").on_press(Message::MoveDown),
            button("Remove (Del)").on_press(Message::RemoveFile)
        ]
        .spacing(SPACING_SM);

        column![
            header,
            horizontal_rule(1),
            scrollable(file_list).height(LIST_HEIGHT),
            horizontal_rule(1),
            controls
        ]
        .spacing(SPACING_MD)
        .width(Fill)
        .into()
    }

    fn view_preview(&self) -> Element<'_, Message> {
        let items: Vec<Element<'_, Message>> = if self.previews.is_empty() {
            vec![text("Preview appears here after configuring options.")
                .size(FONT_SM)
                .into()]
        } else {
            self.previews
                .iter()
                .map(|p| {
                    let conflict = if p.has_conflict {
                        text(" [CONFLICT]").color(COLOR_CONFLICT)
                    } else {
                        text("")
                    };
                    column![
                        text(p.original_name.as_str()).size(FONT_SM),
                        row![
                            text("  -> ").size(FONT_SM).color(COLOR_INFO),
                            text(&p.new_name).size(FONT_SM).color(COLOR_SUCCESS),
                            conflict
                        ]
                    ]
                    .spacing(SPACING_XS)
                    .into()
                })
                .collect()
        };

        column![
            text("Preview").size(FONT_LG),
            horizontal_rule(1),
            scrollable(Column::with_children(items).spacing(8)).height(LIST_HEIGHT)
        ]
        .spacing(SPACING_MD)
        .width(Fill)
        .into()
    }

    fn view_options(&self) -> Element<'_, Message> {
        match self.mode {
            AppMode::FindReplace => self.view_find_replace_options(),
            AppMode::Iteration => self.view_iteration_options(),
        }
    }

    fn view_find_replace_options(&self) -> Element<'_, Message> {
        row![
            column![
                text("Find:").size(FONT_SM),
                text_input("Pattern...", &self.find_pattern)
                    .on_input(Message::FindPatternChanged)
                    .width(250)
            ]
            .spacing(SPACING_SM),
            column![
                text("Replace:").size(FONT_SM),
                text_input("Replacement...", &self.replace_with)
                    .on_input(Message::ReplaceWithChanged)
                    .width(250)
            ]
            .spacing(SPACING_SM),
            column![
                checkbox("Regex", self.regex_mode).on_toggle(Message::RegexModeToggled),
                checkbox("Case Sensitive", self.case_sensitive)
                    .on_toggle(Message::CaseSensitiveToggled)
            ]
            .spacing(SPACING_SM),
            horizontal_space(),
            button(text("Execute (Ctrl+Enter)").size(FONT_LG))
                .on_press(Message::ExecuteRename)
                .style(button::success),
        ]
        .spacing(SPACING_LG)
        .align_y(Center)
        .into()
    }

    fn view_iteration_options(&self) -> Element<'_, Message> {
        row![
            column![
                text("Template ({n}):").size(FONT_SM),
                text_input("photo_{n}", &self.template)
                    .on_input(Message::TemplateChanged)
                    .width(200)
            ]
            .spacing(SPACING_SM),
            column![
                text("Start:").size(FONT_SM),
                text_input("1", &self.start_number)
                    .on_input(Message::StartNumberChanged)
                    .width(80)
            ]
            .spacing(SPACING_SM),
            column![
                text("Padding:").size(FONT_SM),
                text_input("3", &self.padding)
                    .on_input(Message::PaddingChanged)
                    .width(80)
            ]
            .spacing(SPACING_SM),
            horizontal_space(),
            button(text("Execute (Ctrl+Enter)").size(FONT_LG))
                .on_press(Message::ExecuteRename)
                .style(button::success),
        ]
        .spacing(SPACING_LG)
        .align_y(Center)
        .into()
    }

    fn view_status(&self) -> Element<'_, Message> {
        let color = if self.is_error {
            COLOR_ERROR
        } else {
            COLOR_MUTED_DARK
        };
        container(
            text(self.status_message.as_deref().unwrap_or("Ready"))
                .size(FONT_SM)
                .color(color),
        )
        .padding(SPACING_MD)
        .width(Fill)
        .into()
    }
}
