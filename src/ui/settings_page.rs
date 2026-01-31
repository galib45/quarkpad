use std::path::PathBuf;

use adw::subclass::prelude::NavigationPageImpl;
use gtk::gio::prelude::FileExt;
use gtk::glib;
use gtk::glib::object::CastNone;
use gtk::prelude::{ButtonExt, EditableExt, WidgetExt};
use gtk::subclass::prelude::*;

use crate::{models, state};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/galib/quarkpad/ui/settings_page.ui")]
    pub struct QPSettingsPage {
        #[template_child]
        pub proton_path: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub umu_path: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub btn_proton_path: TemplateChild<gtk::Button>,
        #[template_child]
        pub btn_umu_path: TemplateChild<gtk::Button>,
        #[template_child]
        pub btn_save: TemplateChild<adw::ButtonRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QPSettingsPage {
        const NAME: &'static str = "QPSettingsPage";
        type Type = super::QPSettingsPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for QPSettingsPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }
    impl WidgetImpl for QPSettingsPage {}
    impl NavigationPageImpl for QPSettingsPage {}
}

glib::wrapper! {
    pub struct QPSettingsPage(ObjectSubclass<imp::QPSettingsPage>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Orientable;
}

impl QPSettingsPage {
    pub fn new() -> Self {
        let obj: QPSettingsPage = glib::Object::builder().build();
        obj
    }

    fn setup(&self) {
        let imp = self.imp();
        {
            let reader = state().read().unwrap();
            imp.proton_path.set_text(reader.settings.proton_path.to_str().unwrap());
            imp.umu_path.set_text(reader.settings.umu_path.to_str().unwrap());
        }

        imp.btn_save.connect_activated(glib::clone!(
            #[weak(rename_to = imp)] imp,
            move |_| {
                {
                    let mut writer = state().write().unwrap();
                    let settings = models::Settings {
                        proton_path: PathBuf::from(imp.proton_path.text()),
                        umu_path: PathBuf::from(imp.umu_path.text()),
                    };
                    writer.settings = settings;
                    writer.save();
                }
                let obj = imp.obj();
                let nav_view = obj.parent().and_downcast::<adw::NavigationView>().unwrap();
                nav_view.pop();
            }
        ));

        imp.btn_proton_path.connect_clicked(glib::clone!(
            #[weak(rename_to = proton_path)] imp.proton_path,
            move |_btn| {
                let file_dialog = gtk::FileDialog::new();
                file_dialog.select_folder(None::<&gtk::Window>, gtk::gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            proton_path.set_text(path.to_str().unwrap());
                        }
                    }
                });
            }
        ));

        imp.btn_umu_path.connect_clicked(glib::clone!(
            #[weak(rename_to = umu_path)] imp.umu_path,
            move |_btn| {
                let file_dialog = gtk::FileDialog::new();
                file_dialog.select_folder(None::<&gtk::Window>, gtk::gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            umu_path.set_text(path.to_str().unwrap());
                        }
                    }
                });
            }
        ));
    }
}
