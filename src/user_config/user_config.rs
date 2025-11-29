use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::KeyboardMode;

#[derive(Debug)]
pub struct UserConfig {
    pub screenshots_destination_directory: PathBuf,
    pub keyboard_mode: KeyboardMode,
}

impl Default for UserConfig {
    fn default() -> Self {
        let screenshots_destination_subdirectory =
            [OsStr::new("Pictures"), OsStr::new("Screenshots")];
        let screenshots_destination_directory = glib::home_dir()
            .iter()
            .chain(screenshots_destination_subdirectory)
            .collect();

        Self {
            screenshots_destination_directory,
            keyboard_mode: KeyboardMode::default(),
        }
    }
}

impl UserConfig {
    pub fn load() -> Self {
        let mut app_config_path = glib::user_config_dir();
        app_config_path.push("generic_launcher");
        app_config_path.push("config.toml");

        let default_config = Self::default();

        let app_config = match std::fs::read_to_string(app_config_path) {
            Ok(string) => match string.parse::<toml::Table>() {
                Ok(app_config) => app_config,
                Err(e) => {
                    println!("Failed to load config: {}", e.message());
                    return default_config;
                }
            },
            Err(..) => return default_config,
        };

        let screenshots_destination_directory =
            match app_config["screenshots_destination_directory"].as_str() {
                Some(path_string) => Path::new(path_string).to_path_buf(),
                None => default_config.screenshots_destination_directory,
            };

        let keyboard_mode = match app_config["keyboard_mode"].as_str() {
            Some(keyboard_mode) => KeyboardMode::from(keyboard_mode),
            None => default_config.keyboard_mode,
        };

        Self {
            screenshots_destination_directory,
            keyboard_mode,
        }
    }
}
