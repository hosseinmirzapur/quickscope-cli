//! QuickScope Web Frontend — Entry point.

use quickscope_web::App;

fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    leptos::mount::mount_to_body(App);
}
