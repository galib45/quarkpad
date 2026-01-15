use std::path::{Path, PathBuf};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{models, utils};

#[repr(i32)]
enum Page {
    Home,
    Settings,
    AddGame
}

impl crate::App {
    pub fn load_data(&self) {
        let app_data = models::AppData::load();
        let games = app_data.games
            .iter().map(|x| x.to_owned().into())
            .collect::<Vec<crate::Game>>();
        let games_model = ModelRc::new(VecModel::from(games));
        self.set_games(games_model);
        self.set_settings(app_data.settings.into());
    }

    pub fn setup_callbacks(&self) {
        let app_weak = self.as_weak();
        self.on_settings_clicked(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_current_page(Page::Settings as i32);
            }
        });

        let app_weak = self.as_weak();
        self.on_add_fab_clicked(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_game(crate::Game::default());
                app.set_current_page(Page::AddGame as i32);
            }
        });

        let app_weak = self.as_weak();
        self.on_back_clicked(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_current_page(Page::Home as i32);
            }
        });

        let app_weak = self.as_weak();
        self.on_choose_proton_path(move || {
            if let Some(app) = app_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    app.set_proton_path(path);
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_choose_umu_path(move || {
            if let Some(app) = app_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    app.set_umu_path(path);
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_choose_cover_path(move || {
            if let Some(app) = app_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.set_cover_path(path);
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_choose_exe_path(move || {
            if let Some(app) = app_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.set_exe_path(path);
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_choose_wineprefix(move || {
            if let Some(app) = app_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    app.set_wineprefix(path);
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_click_save_settings(move || {
            if let Some(app) = app_weak.upgrade() {
                dbg!(app.get_settings());
            }
        });

        let app_weak = self.as_weak();
        self.on_click_save_game(move || {
            if let Some(app) = app_weak.upgrade() {
                let game = app.get_game();
                let games_model = app.get_games();
                let editing = app.get_editing();
                if let Some(vec_model) = games_model.as_any().downcast_ref::<VecModel<crate::Game>>() {
                    if editing < 0 { vec_model.push(game); }
                    else {
                        vec_model.remove(editing as usize);
                        vec_model.insert(editing as usize, game);
                    }
                    let games = vec_model.iter().map(models::Game::from).collect::<Vec<models::Game>>();
                    let app_data = models::AppData {
                        games, settings: app.get_settings().into()
                    };
                    app_data.save();
                }
                app.set_current_page(Page::Home as i32);
            }
        });

        self.on_load_image(move |path| {
            slint::Image::load_from_path(Path::new(path.as_str())).unwrap()
        });

        let app_weak = self.as_weak();
        self.on_remove_game(move |index| {
            if let Some(app) = app_weak.upgrade() {
                let games_model = app.get_games();
                if games_model.row_data(index as usize).is_some() {
                    if let Some(vec_model) = games_model.as_any().downcast_ref::<VecModel<crate::Game>>() {
                        vec_model.remove(index as usize);
                        let games = vec_model.iter().map(models::Game::from).collect::<Vec<models::Game>>();
                        let app_data = models::AppData {
                            games, settings: app.get_settings().into()
                        };
                        app_data.save();
                    }
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_edit_game(move |index| {
            if let Some(app) = app_weak.upgrade() {
                let games_model = app.get_games();
                if let Some(game) = games_model.row_data(index as usize) {
                    app.set_game(game);
                    app.set_editing(index);
                    app.set_current_page(Page::AddGame as i32);
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_launch_game(move |index| {
            if let Some(app) = app_weak.upgrade() {
                let games = app.get_games();
                let settings = app.get_settings();
                if let Some(game) = games.row_data(index as usize) {
                    utils::launch_game(
                        &models::Game::from(game),
                        &models::Settings::from(settings),
                    );
                }
            }
        });

        let app_weak = self.as_weak();
        self.on_winecfg(move |index| {
            if let Some(app) = app_weak.upgrade() {
                let games = app.get_games();
                let settings = app.get_settings();
                if let Some(game) = games.row_data(index as usize) {
                    utils::launch_winecfg(
                        &models::Game::from(game),
                        &models::Settings::from(settings),
                    );
                }
            }
        });
    }

    fn set_proton_path(&self, path: PathBuf) {
        let mut settings = self.get_settings();
        settings.proton_path = SharedString::from(path.to_str().unwrap());
        self.set_settings(settings);
    }

    fn set_umu_path(&self, path: PathBuf) {
        let mut settings = self.get_settings();
        settings.umu_path = SharedString::from(path.to_str().unwrap());
        self.set_settings(settings);
    }

    fn set_cover_path(&self, path: PathBuf) {
        let mut game = self.get_game();
        game.cover_path = SharedString::from(path.to_str().unwrap());
        self.set_game(game);
    }

    fn set_exe_path(&self, path: PathBuf) {
        let mut game = self.get_game();
        game.exe_path = SharedString::from(path.to_str().unwrap());
        self.set_game(game);
    }

    fn set_wineprefix(&self, path: PathBuf) {
        let mut game = self.get_game();
        game.wineprefix = SharedString::from(path.to_str().unwrap());
        self.set_game(game);
    }
}
