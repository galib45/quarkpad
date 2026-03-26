use gtk::SingleSelection;
use gtk::SignalListItemFactory;
use gtk::prelude::*;
use gtk::glib;
use gtk::gio;
use gtk::glib::object::Cast;
use gtk::glib::object::CastNone;
use gtk::glib::subclass::types::ObjectSubclass;
use adw::subclass::prelude::*;
use gtk::prelude::ListItemExt;

use crate::models;
use crate::models::GameObject;
use crate::state;
use crate::ui::add_edit_game_page::QPAddEditGamePage;
use crate::ui::grid_item::QPGridItem;
use crate::ui::settings_page::QPSettingsPage;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/galib/quarkpad/ui/window.ui")]
    pub struct QPWindow {
        #[template_child]
        pub toasts: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub nav_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub btn_goto_add_edit_page: TemplateChild<gtk::Button>,
        #[template_child]
        pub btn_goto_settings_page: TemplateChild<gtk::Button>,
        #[template_child]
        pub split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub banner: TemplateChild<adw::Banner>,
        #[template_child]
        pub games_grid: TemplateChild<gtk::GridView>,
        #[template_child]
        pub sidebar_label_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub sidebar_label_duration_played: TemplateChild<gtk::Label>,
        #[template_child]
        pub sidebar_label_last_played: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QPWindow {
        const NAME: &'static str = "QPWindow";
        type Type = super::QPWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for QPWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for QPWindow {}
    impl WindowImpl for QPWindow {}
    impl ApplicationWindowImpl for QPWindow {}
    impl AdwApplicationWindowImpl for QPWindow {}
}

glib::wrapper! {
    pub struct QPWindow(ObjectSubclass<imp::QPWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap,
                    gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native,
                    gtk::Root, gtk::ShortcutManager;
}

impl QPWindow {
    pub fn new(app: &adw::Application) -> Self {
        let obj: QPWindow = glib::Object::builder().property("application", app).build();
        obj.setup();
        obj
    }

    fn setup(&self) {
        let imp = self.imp();
        self.refresh();
        imp.banner.connect_button_clicked(glib::clone!(
            #[weak(rename_to = nav_view)] imp.nav_view,
            move |_| {
                let add_page = QPSettingsPage::new();
                nav_view.push(&add_page);
            }
        ));

        imp.btn_goto_add_edit_page.connect_clicked(glib::clone!(
            #[weak(rename_to = nav_view)] imp.nav_view,
            move |_| {
                let add_page = QPAddEditGamePage::new();
                nav_view.push(&add_page);
            }
        ));
        imp.btn_goto_settings_page.connect_clicked(glib::clone!(
            #[weak(rename_to = nav_view)] imp.nav_view,
            move |_| {
                let add_page = QPSettingsPage::new();
                nav_view.push(&add_page);
            }
        ));
        self.setup_games_grid();
        self.setup_actions();

        imp.nav_view.connect_popped(glib::clone!(
            #[weak(rename_to = _self)] self,
            move |_nav_view, _nav_page| {
                _self.refresh();
            }
        ));
    }

    pub fn refresh(&self) {
        let imp = self.imp();

        // refresh the banner
        let settings = {
            let reader = state().read().unwrap();
            reader.settings.clone()
        };
        let mut messages = Vec::new();
        // Check Proton path
        if settings.proton_path.as_os_str().is_empty() {
            messages.push("Proton Path is not set");
        } else if !settings.proton_path.exists() {
            messages.push("Proton Path does not exist");
        }
        // Check UMU path
        if settings.umu_path.as_os_str().is_empty() {
            messages.push("UMU Path is not set");
        } else if !settings.umu_path.exists() {
            messages.push("UMU Path does not exist");
        }
        // Combine messages and update banner
        if messages.is_empty() {
            imp.banner.set_revealed(false);
        } else {
            imp.banner.set_title(&messages.join("; "));
            imp.banner.set_revealed(true);
        }

        // refresh games_grid
        let games = {
            let reader = state().read().unwrap();
            reader.games.clone()
        };

        if let Some(model) = imp.games_grid.model() {
            if let Some(selection) = model.downcast_ref::<SingleSelection>() {
                if let Some(list_store) = selection.model().and_downcast::<gio::ListStore>() {
                    list_store.remove_all();
                    for game in &games {
                        list_store.append(&GameObject::new(game.clone()));
                    }
                }
            }
        }
    }

    fn setup_actions(&self) {
        use gtk::gio;
        let action_group = gio::SimpleActionGroup::new();

        let launch_action = gio::SimpleAction::new("launch", None);
        launch_action.connect_activate(glib::clone!(
            #[weak(rename_to = _self)] self,
            move |_, _| {
                let imp = _self.imp();
                imp.split_view.set_show_sidebar(false);
                let selection_model = imp.games_grid.model().and_downcast::<SingleSelection>().unwrap();
                let index = selection_model.selected();
                let game = {
                    let reader = state().read().unwrap();
                    reader.games[index as usize].clone()
                };
                let settings = {
                    let reader = state().read().unwrap();
                    reader.settings.clone()
                };

                utils::run_with_logs(
                    &_self,
                    game, settings,
                    move |game, settings, mut callback| async move {
                        utils::launch_game(&game, &settings, glib::clone!(
                            move |line| {
                                if line.starts_with("Proton: ") && line.ends_with("exe\n") {
                                    let mut writer = state().write().unwrap();
                                    writer.games[index as usize].last_played = Some(chrono::Utc::now());
                                    writer.save();
                                }
                                if line.starts_with("PROCESS EXITED") {
                                    let now = chrono::Utc::now();
                                    let mut writer = state().write().unwrap();
                                    let matched = writer.games[index as usize].clone();
                                    let duration = now - matched.last_played.unwrap();
                                    writer.games[index as usize].last_played = Some(now);
                                    writer.games[index as usize].duration_played += duration.as_seconds_f64() as u64;
                                    writer.save();
                                }

                                callback(line);
                            }
                        )).await;
                    }
                );
            }
        ));

        let edit_action = gio::SimpleAction::new("edit", None);
        edit_action.connect_activate(glib::clone!(
            #[weak(rename_to = _self)] self,
            move |_, _| {
                let imp = _self.imp();
                imp.split_view.set_show_sidebar(false);
                let selection_model = imp.games_grid.model().and_downcast::<SingleSelection>().unwrap();
                let index = selection_model.selected();
                let game = {
                    let reader = state().read().unwrap();
                    reader.games[index as usize].clone()
                };
                let reader = state().read().unwrap();
                if let Some(game_index) = reader.games.iter().position(|x| *x == game) {
                    let edit_page = QPAddEditGamePage::with_game(game_index);
                    imp.nav_view.push(&edit_page);
                }
            }
        ));

        let remove_action = gio::SimpleAction::new("remove", None);
        remove_action.connect_activate(glib::clone!(
            #[weak(rename_to = _self)] self,
            move |_, _| {
                let imp = _self.imp();
                imp.split_view.set_show_sidebar(false);
                let selection_model = imp.games_grid.model().and_downcast::<SingleSelection>().unwrap();
                let index = selection_model.selected();
                let game = {
                    let reader = state().read().unwrap();
                    reader.games[index as usize].clone()
                };
                {
                    let mut writer = state().write().unwrap();
                    let game_index = writer.games.iter().position(|x| *x == game);
                    if let Some(index) = game_index {
                        writer.games.remove(index);
                        writer.save();
                    }
                }
                _self.refresh();
            }
        ));

        let winecfg_action = gio::SimpleAction::new("winecfg", None);
        winecfg_action.connect_activate(glib::clone!(
            #[weak(rename_to = _self)] self,
            move |_, _| {
                let imp = _self.imp();
                imp.split_view.set_show_sidebar(false);
                let selection_model = imp.games_grid.model().and_downcast::<SingleSelection>().unwrap();
                let index = selection_model.selected();
                let game = {
                    let reader = state().read().unwrap();
                    reader.games[index as usize].clone()
                };
                let settings = {
                    let reader = state().read().unwrap();
                    reader.settings.clone()
                };
                utils::run_with_logs(
                    &_self,
                    game, settings,
                    |game, settings, callback| async move {
                        utils::launch_winecfg(&game, &settings, callback).await;
                    }
                );
            }
        ));

        action_group.add_action(&launch_action);
        action_group.add_action(&edit_action);
        action_group.add_action(&remove_action);
        action_group.add_action(&winecfg_action);
        self.insert_action_group("sidebar", Some(&action_group));
    }

    fn show_sidebar(&self, index: usize) {
        let game = {
            let reader = state().read().unwrap();
            reader.games[index as usize].clone()
        };
        let imp = self.imp();
        imp.sidebar_label_title.set_text(&game.name);
        let duration_played_text = if game.duration_played > 0 {
            format!("Played for {}", utils::human_readable_duration(game.duration_played))
        } else {
            "Not played yet".into()
        };
        imp.sidebar_label_duration_played.set_text(
            &duration_played_text
        );

        if let Some(last_played) = game.last_played {
            let last_played_text = format!("Last played {}", utils::time_ago(last_played));
            imp.sidebar_label_last_played.set_text(&last_played_text);
        } else {
            imp.sidebar_label_last_played.set_text("");
        }
        imp.split_view.set_show_sidebar(true);
    }

    fn setup_games_grid(&self) {
        let imp = self.imp();
        let model = gio::ListStore::new::<models::GameObject>();
        {
            let reader = state().read().unwrap();
            let games = reader.games.clone();
            for game in games {
                model.append(&GameObject::new(game));
            }
        }
        let selection_model = SingleSelection::new(None::<gio::ListModel>);
        selection_model.set_autoselect(false);
        selection_model.set_model(Some(&model));
        let factory = SignalListItemFactory::new();

        factory.connect_setup(glib::clone!(
            move |_factory, item| {
                let list_item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let qp_grid_item = QPGridItem::new();
                list_item.set_child(Some(&qp_grid_item));
            }
        ));
        factory.connect_bind(glib::clone!(
            #[weak(rename_to = nav_view)] imp.nav_view,
            move |_factory, item| {
                let list_item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let game_object = list_item.item().and_downcast::<models::GameObject>().unwrap();
                if let Some(game) = game_object.game() {
                    let qp_grid_item = list_item.child().and_downcast::<QPGridItem>().unwrap();
                    qp_grid_item.bind(&game, &nav_view);
                }
            }
        ));
        imp.games_grid.set_model(Some(&selection_model));
        imp.games_grid.set_factory(Some(&factory));
        imp.games_grid.set_min_columns(2);
        imp.games_grid.set_max_columns(6);
        imp.games_grid.set_single_click_activate(true);
        imp.games_grid.connect_activate(glib::clone!(
            #[weak(rename_to = _self)] self,
            move |_grid_view, index| {
                _self.show_sidebar(index as usize);
            }
        ));
    }
}
