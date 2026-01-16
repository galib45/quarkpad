slint::include_modules!();

mod models;
mod utils;
mod main_window;

fn main() {
    let main_window = MainWindow::new().unwrap();

    main_window.load_data();
    main_window.setup_callbacks();
    main_window.run().unwrap();
}
