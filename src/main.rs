slint::include_modules!();

mod models;
mod utils;
mod app;

fn main() {
    smol::block_on(async {
        let app = App::new().unwrap();
        app.load_data();
        app.setup_callbacks();
        app.run().unwrap();
    });
}
