use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::RwLock;

use gtk::prelude::*;
use gtk::glib;
use gtk::gio;

use crate::models::AppData;
use crate::ui::window::QPWindow;

mod ui;
mod models;
mod utils;

const APP_ID: &str = "org.galib.quarkpad";
static STATE: OnceLock<Arc<RwLock<AppData>>> = OnceLock::new();

pub fn state() -> &'static Arc<RwLock<AppData>> {
    STATE.get_or_init(|| {
        Arc::new(RwLock::new(AppData::load()))
    })
}

fn main() -> glib::ExitCode {
    gio::resources_register_include!("compiled.gresource").expect("Failed to register resource");
    unsafe {
        glib::setenv("GSK_RENDERER", "gl", true).unwrap();
    }
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(app_main);
    app.run()
}

fn app_main(app: &adw::Application) {
    load_custom_css();
    let window = QPWindow::new(app);
    window.present();
}

fn load_custom_css() {
    let css = gtk::CssProvider::new();
    css.load_from_string(r#"
        gridview {
            background-color: transparent;
        }
        textview {
            font-family: monospace;
        }
    "#);

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
