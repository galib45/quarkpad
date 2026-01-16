use std::path::{Path, PathBuf};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{models, utils};

#[repr(i32)]
enum Page {
    Home,
    Settings,
    AddGame
}

impl crate::MainWindow {
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
        let self_weak = self.as_weak();
        self.on_settings_clicked(move || {
            if let Some(main_window) = self_weak.upgrade() {
                main_window.set_current_page(Page::Settings as i32);
            }
        });

        let self_weak = self.as_weak();
        self.on_add_fab_clicked(move || {
            if let Some(main_window) = self_weak.upgrade() {
                main_window.set_game(crate::Game::default());
                main_window.set_current_page(Page::AddGame as i32);
            }
        });

        let self_weak = self.as_weak();
        self.on_back_clicked(move || {
            if let Some(main_window) = self_weak.upgrade() {
                main_window.set_current_page(Page::Home as i32);
            }
        });

        let self_weak = self.as_weak();
        self.on_choose_proton_path(move || {
            if let Some(main_window) = self_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    main_window.set_proton_path(path);
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_choose_umu_path(move || {
            if let Some(main_window) = self_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    main_window.set_umu_path(path);
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_choose_cover_path(move || {
            if let Some(main_window) = self_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    main_window.set_cover_path(path);
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_choose_exe_path(move || {
            if let Some(main_window) = self_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    main_window.set_exe_path(path);
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_choose_wineprefix(move || {
            if let Some(main_window) = self_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    main_window.set_wineprefix(path);
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_click_save_settings(move || {
            if let Some(main_window) = self_weak.upgrade() {
                dbg!(main_window.get_settings());
            }
        });

        let self_weak = self.as_weak();
        self.on_click_save_game(move || {
            if let Some(main_window) = self_weak.upgrade() {
                let game = main_window.get_game();
                let games_model = main_window.get_games();
                let editing = main_window.get_editing();
                if let Some(vec_model) = games_model.as_any().downcast_ref::<VecModel<crate::Game>>() {
                    if editing < 0 { vec_model.push(game); }
                    else {
                        vec_model.remove(editing as usize);
                        vec_model.insert(editing as usize, game);
                    }
                    let games = vec_model.iter().map(models::Game::from).collect::<Vec<models::Game>>();
                    let app_data = models::AppData {
                        games, settings: main_window.get_settings().into()
                    };
                    app_data.save();
                }
                main_window.set_current_page(Page::Home as i32);
            }
        });

        self.on_load_image(move |path| {
            slint::Image::load_from_path(Path::new(path.as_str())).unwrap()
        });

        let self_weak = self.as_weak();
        self.on_remove_game(move |index| {
            if let Some(main_window) = self_weak.upgrade() {
                let games_model = main_window.get_games();
                if games_model.row_data(index as usize).is_some() {
                    if let Some(vec_model) = games_model.as_any().downcast_ref::<VecModel<crate::Game>>() {
                        vec_model.remove(index as usize);
                        let games = vec_model.iter().map(models::Game::from).collect::<Vec<models::Game>>();
                        let app_data = models::AppData {
                            games, settings: main_window.get_settings().into()
                        };
                        app_data.save();
                    }
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_edit_game(move |index| {
            if let Some(main_window) = self_weak.upgrade() {
                let games_model = main_window.get_games();
                if let Some(game) = games_model.row_data(index as usize) {
                    main_window.set_game(game);
                    main_window.set_editing(index);
                    main_window.set_current_page(Page::AddGame as i32);
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_launch_game(move |index| {
            if let Some(main_window) = self_weak.upgrade() {
                let games = main_window.get_games();
                let settings = main_window.get_settings();
                main_window.set_show_logs_dialog(true);
                let self_weak = main_window.as_weak();
                if let Some(game) = games.row_data(index as usize) {
                    smol::spawn(async move {
                        utils::launch_game(
                            &models::Game::from(game),
                            &models::Settings::from(settings),
                            move |line| {
                                let self_weak = self_weak.clone();
                                slint::invoke_from_event_loop(move || {
                                    if let Some(main_window) = self_weak.upgrade() {
                                        let mut content = main_window.get_logs_content();
                                        content.push_str(&line);
                                        main_window.set_logs_content(content);
                                    }
                                }).unwrap();
                            }
                        ).await;
                    }).detach();
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_winecfg(move |index| {
            if let Some(main_window) = self_weak.upgrade() {
                let games = main_window.get_games();
                let settings = main_window.get_settings();
                main_window.set_show_logs_dialog(true);
                let self_weak = main_window.as_weak();
                if let Some(game) = games.row_data(index as usize) {
                    smol::spawn(async move {
                        utils::launch_winecfg(
                            &models::Game::from(game),
                            &models::Settings::from(settings),
                            move |line| {
                                let self_weak = self_weak.clone();
                                slint::invoke_from_event_loop(move || {
                                    if let Some(main_window) = self_weak.upgrade() {
                                        let mut content = main_window.get_logs_content();
                                        content.push_str(&line);
                                        main_window.set_logs_content(content);
                                    }
                                }).unwrap();
                            }
                        ).await;
                    }).detach();
                }
            }
        });

        let self_weak = self.as_weak();
        self.on_logs_dialog_close_clicked(move || {
            if let Some(main_window) = self_weak.upgrade() {
                main_window.set_show_logs_dialog(false);
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
