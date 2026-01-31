use gtk::glib;
use gtk::prelude::{TextBufferExt, TextViewExt};
use gtk::subclass::prelude::*;

mod imp {
    use adw::subclass::dialog::AdwDialogImpl;

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/galib/quarkpad/ui/logs_dialog.ui")]
    pub struct LogsDialog {
        #[template_child]
        pub logs_view: TemplateChild<gtk::TextView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LogsDialog {
        const NAME: &'static str = "LogsDialog";
        type Type = super::LogsDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LogsDialog {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for LogsDialog {}
    impl AdwDialogImpl for LogsDialog {}
}

glib::wrapper! {
    pub struct LogsDialog(ObjectSubclass<imp::LogsDialog>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Orientable;
}

impl LogsDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn append_line(&self, line: String) {
        let imp = self.imp();
        let buffer = imp.logs_view.buffer();
        let mut end = buffer.end_iter();
        buffer.insert(&mut end, &line);
        imp.logs_view.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
    }
}
