//! Entry point for the Leptos WASM frontend.
//!
//! Mounts the App component into the DOM.

use leptos::prelude::*;

use automapper_web::app::App;

fn main() {
    // Set up panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();

    // Initialize console logging
    _ = console_log::init_with_level(log::Level::Debug);

    log::info!("automapper-web starting");

    mount_to_body(App);
}
