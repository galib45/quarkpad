use chrono::Utc;
use gtk::gio::prelude::ActionMapExt;
use gtk::glib;
use gtk::glib::object::CastNone;
use gtk::prelude::{GestureSingleExt, PopoverExt, WidgetExt};
use gtk::subclass::prelude::*;

use crate::models::Game;
use crate::ui::add_edit_game_page::QPAddEditGamePage;
use crate::ui::window::QPWindow;
use crate::{state, utils};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/galib/quarkpad/ui/grid_item.ui")]
    pub struct QPGridItem {
        #[template_child]
        pub cover_image: TemplateChild<gtk::Picture>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub popover_menu: TemplateChild<gtk::PopoverMenu>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QPGridItem {
        const NAME: &'static str = "QPGridItem";
        type Type = super::QPGridItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for QPGridItem {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for QPGridItem {}
    impl BoxImpl for QPGridItem {}
}

glib::wrapper! {
    pub struct QPGridItem(ObjectSubclass<imp::QPGridItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Orientable;
}

impl QPGridItem {
    pub fn new() -> Self {
        let obj: QPGridItem = glib::Object::builder().build();
        obj
    }

    pub fn bind(&self, game: &Game, nav_view: &adw::NavigationView) {
        let imp = self.imp();
        imp.cover_image.set_filename(Some(&game.cover_path));
        imp.label.set_text(&game.name);

        let gesture = gtk::GestureClick::new();
        gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
        gesture.connect_released(glib::clone!(
            #[weak(rename_to = popover_menu)] imp.popover_menu,
            move |_, _, x, y| {
                popover_menu.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 0, 0)));
                popover_menu.popup();
            }
        ));
        self.add_controller(gesture);
        use gtk::gio;
        let action_group = gio::SimpleActionGroup::new();

        let launch_action = gio::SimpleAction::new("launch", None);
        launch_action.connect_activate(glib::clone!(
            #[strong] game,
            #[weak(rename_to = _self)] self,
            move |_, _| {
                let game = game.clone();
                let settings = {
                    let reader = state().read().unwrap();
                    reader.settings.clone()
                };
                utils::run_with_logs(
                    &_self.root().unwrap(),
                    game, settings,
                    |game, settings, mut callback| async move {
                        utils::launch_game(&game, &settings, glib::clone!(
                            #[strong] game,
                            move |line| {
                                if line.starts_with("Proton: ") && line.ends_with("exe\n") {
                                    let mut writer = state().write().unwrap();
                                    if let Some(matched) = writer.games.iter_mut().find(|g| **g == game) {
                                        matched.last_played = Some(Utc::now());
                                        writer.save();
                                    }
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
            #[strong] game,
            #[weak] nav_view,
            move |_, _| {
                let reader = state().read().unwrap();
                if let Some(game_index) = reader.games.iter().position(|x| *x == game) {
                    let edit_page = QPAddEditGamePage::with_game(game_index);
                    nav_view.push(&edit_page);
                }
            }
        ));

        let remove_action = gio::SimpleAction::new("remove", None);
        remove_action.connect_activate(glib::clone!(
            #[strong] game,
            #[weak(rename_to = _self)] self,
            move |_, _| {
                {
                    let mut writer = state().write().unwrap();
                    let game_index = writer.games.iter().position(|x| *x == game);
                    if let Some(index) = game_index {
                        writer.games.remove(index);
                        writer.save();
                    }
                }
                let qp_window = _self.root().and_downcast::<QPWindow>().unwrap();
                qp_window.refresh();
            }
        ));

        let winecfg_action = gio::SimpleAction::new("winecfg", None);
        winecfg_action.connect_activate(glib::clone!(
            #[strong] game,
            #[weak(rename_to = _self)] self,
            move |_, _| {
                let reader = state().read().unwrap();
                let game = game.clone();
                let settings = reader.settings.clone();
                utils::run_with_logs(
                    &_self.root().unwrap(),
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
        self.insert_action_group("item", Some(&action_group));
    }
}
