use std::{fs, path::PathBuf};

use slint::SharedString;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub proton_path: PathBuf,
    pub umu_path: PathBuf,
}

impl From<crate::Settings> for Settings {
    fn from(settings: crate::Settings) -> Self {
        Self {
            proton_path: PathBuf::from(settings.proton_path.as_str()),
            umu_path: PathBuf::from(settings.umu_path.as_str())
        }
    }
}

impl Into<crate::Settings> for Settings {
    fn into(self) -> crate::Settings {
        crate::Settings {
            proton_path: SharedString::from(self.proton_path.to_str().unwrap()),
            umu_path: SharedString::from(self.umu_path.to_str().unwrap())
        }
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Game {
    pub name: String,
    pub cover_path: PathBuf,
    pub exe_path: PathBuf,
    pub wineprefix: PathBuf,
    #[serde(default)]
    pub use_gamescope: bool,
    #[serde(default = "default_gamescope_width")]
    pub gamescope_width: i32,
    #[serde(default = "default_gamescope_height")]
    pub gamescope_height: i32,
}

fn default_gamescope_width() -> i32 { 1920 }
fn default_gamescope_height() -> i32 { 1080 }

impl From<crate::Game> for Game {
    fn from(game: crate::Game) -> Self {
        Self {
            name: game.name.to_string(),
            cover_path: PathBuf::from(game.cover_path.as_str()),
            exe_path: PathBuf::from(game.exe_path.as_str()),
            wineprefix: PathBuf::from(game.wineprefix.as_str()),
            use_gamescope: game.use_gamescope,
            gamescope_width: game.gamescope_width,
            gamescope_height: game.gamescope_height,
        }
    }
}

impl Into<crate::Game> for Game {
    fn into(self) -> crate::Game {
        crate::Game {
            name: SharedString::from(self.name.as_str()),
            cover_path: SharedString::from(self.cover_path.to_str().unwrap()),
            exe_path: SharedString::from(self.exe_path.to_str().unwrap()),
            wineprefix: SharedString::from(self.wineprefix.to_str().unwrap()),
            use_gamescope: self.use_gamescope,
            gamescope_width: self.gamescope_width,
            gamescope_height: self.gamescope_height,
        }
    }
}

pub const APP_ID: &str = "quarkpad";

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppData {
    #[serde(default)]
    pub games: Vec<Game>,
    #[serde(default)]
    pub settings: Settings,
}

impl AppData {
    pub fn load() -> Self {
        if let Some(data_dir) = dirs::data_local_dir() {
            let app_data_dir = data_dir.join(APP_ID);
            if !app_data_dir.exists() {
                fs::create_dir_all(&app_data_dir).unwrap();
            }
            let data_file_path = app_data_dir.join("data.toml");
            let contents = fs::read_to_string(&data_file_path).unwrap_or_default();
            let app_data = toml::from_str::<Self>(&contents).unwrap();
            app_data
        } else { Self::default() }
    }

    pub fn save(&self) {
        if let Some(data_dir) = dirs::data_local_dir() {
            let app_data_dir = data_dir.join(APP_ID);
            if !app_data_dir.exists() {
                fs::create_dir_all(&app_data_dir).unwrap();
            }
            let data_file_path = app_data_dir.join("data.toml");
            fs::write(&data_file_path, toml::to_string_pretty(&self).unwrap()).unwrap();
        }
    }
}
