use gtk::NoSelection;
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
        pub games_grid: TemplateChild<gtk::GridView>,
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
        imp.btn_goto_add_edit_page.connect_clicked(glib::clone!(
            #[weak(rename_to = nav_view)] imp.nav_view,
            move |_| {
                let add_page = QPAddEditGamePage::new();
                nav_view.push(&add_page);
            }
        ));
        self.setup_games_grid();
        imp.nav_view.connect_popped(glib::clone!(
            #[weak(rename_to = _self)] self,
            move |_nav_view, _nav_page| {
                _self.refresh_games_grid();
            }
        ));
    }

    pub fn refresh_games_grid(&self) {
        let imp = self.imp();

        let games = {
            let reader = state().read().unwrap();
            reader.games.clone()
        };

        if let Some(model) = imp.games_grid.model() {
            if let Some(selection) = model.downcast_ref::<NoSelection>() {
                if let Some(list_store) = selection.model().and_downcast::<gio::ListStore>() {
                    list_store.remove_all();
                    for game in &games {
                        list_store.append(&GameObject::new(game.clone()));
                    }
                }
            }
        }
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
        let selection_model = NoSelection::new(Some(model.clone()));
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
        imp.games_grid.connect_activate(glib::clone!(
            move |grid_view, index| {
                let model = grid_view.model().unwrap();
                let game_object = model.item(index).and_downcast::<models::GameObject>().unwrap();
                if let Some(game) = game_object.game() {
                    let settings = {
                        let reader = state().read().unwrap();
                        reader.settings.clone()
                    };
                    smol::spawn(async move {
                        utils::launch_game(
                            &game,
                            &settings,
                            move |line| {
                                eprint!("{line}");
                            }
                        ).await;
                    }).detach();
                }
            }
        ));
    }
}
