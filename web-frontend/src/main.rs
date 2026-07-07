//! QuickScope Web Frontend — Entry point.

use quickscope_web::App;

fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    leptos::mount_to_body(|| leptos::view! { <App /> });
}
