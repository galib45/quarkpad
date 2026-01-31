use std::path::PathBuf;
use std::cell::RefCell;

use adw::subclass::prelude::NavigationPageImpl;
use adw::prelude::PreferencesGroupExt;
use gtk::gio::prelude::FileExt;
use gtk::glib;
use gtk::glib::object::CastNone;
use gtk::prelude::{ButtonExt, EditableExt, WidgetExt};
use gtk::subclass::prelude::*;

use crate::models::GamescopeConfig;
use crate::{models, state, utils};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/galib/quarkpad/ui/add_edit_game_page.ui")]
    pub struct QPAddEditGamePage {
        #[template_child]
        pub game_name: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub cover_path: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub exe_path: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub wineprefix_path: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub btn_cover_path: TemplateChild<gtk::Button>,
        #[template_child]
        pub btn_exe_path: TemplateChild<gtk::Button>,
        #[template_child]
        pub btn_wineprefix_path: TemplateChild<gtk::Button>,
        #[template_child]
        pub extra_args: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub use_gamescope: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub pref_grp_gamescope: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub gamescope_width: TemplateChild<gtk::Entry>,
        #[template_child]
        pub gamescope_height: TemplateChild<gtk::Entry>,
        #[template_child]
        pub btn_save: TemplateChild<adw::ButtonRow>,

        pub game_index: RefCell<Option<usize>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QPAddEditGamePage {
        const NAME: &'static str = "QPAddEditGamePage";
        type Type = super::QPAddEditGamePage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for QPAddEditGamePage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }
    impl WidgetImpl for QPAddEditGamePage {}
    impl NavigationPageImpl for QPAddEditGamePage {}
}

glib::wrapper! {
    pub struct QPAddEditGamePage(ObjectSubclass<imp::QPAddEditGamePage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Orientable;
}

impl QPAddEditGamePage {
    pub fn new() -> Self {
        let obj: QPAddEditGamePage = glib::Object::builder().build();
        obj
    }

    pub fn with_game(game_index: usize) -> Self {
        let obj: QPAddEditGamePage = glib::Object::builder().build();
        let imp = obj.imp();
        *imp.game_index.borrow_mut() = Some(game_index);
        let reader = state().read().unwrap();
        let game = &reader.games[game_index];
        imp.game_name.set_text(&game.name);
        imp.cover_path.set_text(game.cover_path.to_str().unwrap());
        imp.exe_path.set_text(game.exe_path.to_str().unwrap());
        imp.wineprefix_path.set_text(game.wineprefix_path.to_str().unwrap());
        imp.extra_args.set_text(&game.extra_args);
        if let Some(config) = &game.gamescope_config {
            imp.use_gamescope.set_active(true);
            imp.gamescope_width.set_text(&config.output_width.to_string());
            imp.gamescope_height.set_text(&config.output_height.to_string());
        } else {
            imp.use_gamescope.set_active(false);
        }
        obj
    }

    fn setup(&self) {
        let imp = self.imp();
        if !utils::command_exists("gamescope") {
            imp.pref_grp_gamescope.set_description(Some("Gamescope not found"));
        }

        imp.btn_save.connect_activated(glib::clone!(
            #[weak(rename_to = imp)] imp,
            move |_| {
                {
                    let mut writer = state().write().unwrap();
                    let game = models::Game {
                        name: imp.game_name.text().to_string(),
                        cover_path: PathBuf::from(imp.cover_path.text()),
                        exe_path: PathBuf::from(imp.exe_path.text()),
                        wineprefix_path: PathBuf::from(imp.wineprefix_path.text()),
                        extra_args: imp.extra_args.text().to_string(),
                        gamescope_config: if imp.use_gamescope.is_active() {
                            Some(GamescopeConfig{
                                output_width: imp.gamescope_width.text().parse().unwrap_or(1920),
                                output_height: imp.gamescope_height.text().parse().unwrap_or(1080)
                            })
                            } else {
                                None
                            }
                    };
                    if let Some(index) = *imp.game_index.borrow() {
                        writer.games[index] = game;
                    } else {
                        writer.games.push(game);
                    }
                    writer.save();
                }
                let obj = imp.obj();
                let nav_view = obj.parent().and_downcast::<adw::NavigationView>().unwrap();
                nav_view.pop();
            }
        ));

        imp.btn_cover_path.connect_clicked(glib::clone!(
            #[weak(rename_to = cover_path)] imp.cover_path,
            move |_btn| {
                let file_dialog = gtk::FileDialog::new();
                file_dialog.open(None::<&gtk::Window>, gtk::gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            cover_path.set_text(path.to_str().unwrap());
                        }
                    }
                });
            }
        ));

        imp.btn_exe_path.connect_clicked(glib::clone!(
            #[weak(rename_to = exe_path)] imp.exe_path,
            move |_btn| {
                let file_dialog = gtk::FileDialog::new();
                file_dialog.open(None::<&gtk::Window>, gtk::gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            exe_path.set_text(path.to_str().unwrap());
                        }
                    }
                });
            }
        ));

        imp.btn_wineprefix_path.connect_clicked(glib::clone!(
            #[weak(rename_to = wineprefix_path)] imp.wineprefix_path,
            move |_btn| {
                let file_dialog = gtk::FileDialog::new();
                file_dialog.select_folder(None::<&gtk::Window>, gtk::gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            wineprefix_path.set_text(path.to_str().unwrap());
                        }
                    }
                });
            }
        ));

        imp.use_gamescope.connect_active_notify(glib::clone!(
            #[weak(rename_to = pref_grp_gamescope)] imp.pref_grp_gamescope,
            move |switch_row| {
                pref_grp_gamescope.set_visible(switch_row.is_active());
            }
        ));
    }
}
