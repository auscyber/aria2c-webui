use std::sync::Arc;

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
use leptos::prelude::*;

pub mod app;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

#[cfg(feature = "ssr")]
#[derive(FromRef,  Clone)]
pub struct AppState {
    pub aria2: Arc<tokio::sync::RwLock<aria2_ws::Client>>,
    pub leptos_options: LeptosOptions,
}