// Settings persistence using SQLite

use crate::theme::{MAX_PATTERN_LENGTH, MAX_TEMPLATE_LENGTH};
use rusqlite::{Connection, Result as SqlResult};
use std::path::PathBuf;

pub struct Settings {
    pub dark_mode: bool,
    pub regex_mode: bool,
    pub case_sensitive: bool,
    pub template: String,
    pub start_number: u32,
    pub padding: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            regex_mode: false,
            case_sensitive: true,
            template: String::from("{n}"),
            start_number: 1,
            padding: 3,
        }
    }
}

impl Settings {
    // Validates and sanitizes settings values
    #[allow(dead_code)]
    pub fn sanitize(&mut self) {
        if self.template.len() > MAX_TEMPLATE_LENGTH {
            self.template.truncate(MAX_TEMPLATE_LENGTH);
        }
        if self.padding > 10 {
            self.padding = 10;
        }
    }
}

// Returns path to settings database
fn get_db_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("file-rename-plus").join("settings.db"))
}

// Initializes database and creates tables if needed
fn init_db(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

// Loads settings from SQLite database
pub fn load_settings() -> Settings {
    let Some(db_path) = get_db_path() else {
        return Settings::default();
    };

    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let Ok(conn) = Connection::open(&db_path) else {
        return Settings::default();
    };
    if init_db(&conn).is_err() {
        return Settings::default();
    }

    let mut settings = Settings::default();

    if let Ok(val) = get_setting(&conn, "dark_mode") {
        settings.dark_mode = val == "true";
    }
    if let Ok(val) = get_setting(&conn, "regex_mode") {
        settings.regex_mode = val == "true";
    }
    if let Ok(val) = get_setting(&conn, "case_sensitive") {
        settings.case_sensitive = val == "true";
    }
    if let Ok(val) = get_setting(&conn, "template") {
        settings.template = val.chars().take(MAX_TEMPLATE_LENGTH).collect();
    }
    if let Ok(val) = get_setting(&conn, "start_number") {
        settings.start_number = val.parse().unwrap_or(1);
    }
    if let Ok(val) = get_setting(&conn, "padding") {
        settings.padding = val.parse().unwrap_or(3).min(10);
    }

    settings
}

// Saves settings to SQLite database (call from async context)
pub fn save_settings(settings: &Settings) {
    let Some(db_path) = get_db_path() else { return };

    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let Ok(conn) = Connection::open(&db_path) else {
        return;
    };
    if init_db(&conn).is_err() {
        return;
    }

    // Validate before saving
    let template: String = settings
        .template
        .chars()
        .take(MAX_TEMPLATE_LENGTH)
        .collect();

    let _ = set_setting(&conn, "dark_mode", &settings.dark_mode.to_string());
    let _ = set_setting(&conn, "regex_mode", &settings.regex_mode.to_string());
    let _ = set_setting(
        &conn,
        "case_sensitive",
        &settings.case_sensitive.to_string(),
    );
    let _ = set_setting(&conn, "template", &template);
    let _ = set_setting(&conn, "start_number", &settings.start_number.to_string());
    let _ = set_setting(&conn, "padding", &settings.padding.min(10).to_string());
}

fn get_setting(conn: &Connection, key: &str) -> SqlResult<String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
        row.get(0)
    })
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> SqlResult<()> {
    // Limit value length
    let safe_value: String = value.chars().take(MAX_PATTERN_LENGTH).collect();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [key, &safe_value],
    )?;
    Ok(())
}
