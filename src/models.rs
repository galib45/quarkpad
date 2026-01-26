use std::{fs, path::PathBuf};

use gtk::glib;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub proton_path: PathBuf,
    pub umu_path: PathBuf,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Game {
    pub name: String,
    pub cover_path: PathBuf,
    pub exe_path: PathBuf,
    pub wineprefix_path: PathBuf,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub extra_args: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamescope_config: Option<GamescopeConfig>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GamescopeConfig {
    pub output_width: usize,
    pub output_height: usize,
}

use adw::subclass::prelude::*;

mod imp {
    use std::cell::RefCell;
    use super::*;

    #[derive(Default)]
    pub struct GameObject {
        pub game: RefCell<Option<Game>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GameObject {
        const NAME: &'static str = "GameObject";
        type Type = super::GameObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for GameObject {}
}

glib::wrapper! {
    pub struct GameObject(ObjectSubclass<imp::GameObject>);
}

impl GameObject {
    pub fn new(game: Game) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().game.replace(Some(game));
        obj
    }

    pub fn game(&self) -> Option<Game> {
        self.imp().game.borrow().clone()
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppData {
    #[serde(default)]
    pub games: Vec<Game>,
    #[serde(default)]
    pub settings: Settings,
}

const APP_NAME: &str = "quarkpad";

impl AppData {
    pub fn load() -> Self {
        if let Some(data_dir) = dirs::data_local_dir() {
            let app_data_dir = data_dir.join(APP_NAME);
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
            let app_data_dir = data_dir.join(APP_NAME);
            if !app_data_dir.exists() {
                fs::create_dir_all(&app_data_dir).unwrap();
            }
            let data_file_path = app_data_dir.join("data.toml");
            fs::write(&data_file_path, toml::to_string_pretty(&self).unwrap()).unwrap();
        }
    }
}
