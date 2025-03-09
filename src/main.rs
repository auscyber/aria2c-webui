mod types;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use aria2_leptos::{app::*, AppState};
    use axum::{extract::{Path, Request, State}, http::HeaderMap, response::IntoResponse, routing::{get, post}, Router};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, generate_route_list_with_exclusions_and_ssg_and_context, handle_server_fns_with_context, LeptosRoutes};
    #[cfg(feature = "ssr")]
use axum::response::Response as AxumResponse;
    use leptos_ws::server_signals;


      async fn server_fn_handler(
        State(state): State<AppState>,
        _path: Path<String>,
        _headers: HeaderMap,
        _query: axum::extract::RawQuery,
        request: Request,
    ) -> impl IntoResponse {
        handle_server_fns_with_context(
            move || {
                provide_context(state.leptos_options.clone());
                provide_context(state.server_signals.clone());
                provide_context(state.aria2.clone());
                provide_context(state.clone());
            },
            request,
        )
        .await
    }
    async fn leptos_routes_handler(state: State<AppState>, req: Request) -> AxumResponse {
        let state1 = state.0.clone();
        let options2 = state.clone().0.leptos_options.clone();
        let handler = leptos_axum::render_route_with_context(
            state.routes.clone().unwrap(),
            move || {
                provide_context(state1.leptos_options.clone());
                provide_context(state1.server_signals.clone());
            },
            move || shell(options2.clone()),
        );
        handler(state, req).await.into_response()
    }


    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let server_signals = server_signals::ServerSignals::new();

    // Generate the list of routes in your Leptos App
    let aria2 = Arc::new(tokio::sync::RwLock::new(
        aria2_ws::Client::connect("ws://localhost:6800/jsonrpc", None)
            .await
            .unwrap(),
    ));
    let mut app_state = AppState {
        aria2: aria2.clone(),
        server_signals: server_signals.clone(),
        routes: None,
        leptos_options: leptos_options.clone(),
    };
    let state2= app_state.clone();

    let (routes,_) = generate_route_list_with_exclusions_and_ssg_and_context(
        || view! { <App/> }, None,  move || {
            provide_context(state2.server_signals.clone());
            provide_context(state2.clone());
});
    app_state.routes = Some(routes.clone());
    let app = Router::new()
    .route("/api/*fn_name", post(server_fn_handler))
    .route("/ws", get(leptos_ws::axum::websocket(app_state.server_signals.clone())))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
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
