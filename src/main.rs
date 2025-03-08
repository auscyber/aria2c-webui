
mod types;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use aria2_leptos::{app::*, AppState};

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    let aria2 = Arc::new(tokio::sync::RwLock::new(aria2_ws::Client::connect("ws://localhost:6800/jsonrpc", None).await.unwrap()));
    let app_state = AppState {
        aria2: aria2.clone(),
        leptos_options: leptos_options.clone(),
    };
    let app = Router::new()
        .leptos_routes_with_context(&app_state, routes, {
            let aria2a = aria2.clone();
            move || {
                provide_context(aria2a.clone());
            }

        },
    move || shell(leptos_options.clone()))
        .fallback(leptos_axum::file_and_error_handler::<AppState,_>(shell))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
