use std::sync::Arc;


use leptos::{prelude::*, tachys::view, task::spawn_local};

use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}



use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LinkData {
    gid: String,
    total_length: u64,
    completed_length: u64,
    download_speed: u64,
    upload_speed: u64,
    connections: u64,
    num_seeders: Option<u64>,
    num_pieces: u64,
    piece_length: u64,
    dir: String,
    files: Vec<String>,
}


#[server]
async fn fetch_links() -> Result<Vec<LinkData>, server_fn::error::ServerFnError> {
    use crate::AppState;
    let state =  expect_context::<AppState>();
    let client = state.aria2.read().await;
    let b = client.tell_active().await?;
    let c = client.tell_waiting(0, 1000).await?;
    let d = client.tell_stopped(0, 1000).await?;
    let b = b.iter().chain(c.iter()).chain(d.iter()).collect::<Vec<_>>();
    Ok(b.iter().map(|x| LinkData {
        gid: x.gid.clone(),
        total_length: x.total_length.clone(),
        completed_length: x.completed_length.clone(),
        download_speed: x.download_speed.clone(),
        upload_speed: x.upload_speed.clone(),
        connections: x.connections.clone(),
        num_seeders: x.num_seeders.clone(),
        num_pieces: x.num_pieces.clone(),
        piece_length: x.piece_length.clone(),
        dir: x.dir.clone(),
        files: x.files.iter().map(|x| x.path.clone()).collect()
    }).collect())
}

#[server]
async fn add_link(link: String) -> Result<(), server_fn::error::ServerFnError> {

    use crate::AppState;
    let state = expect_context::<AppState>();
    let client = state.aria2.read().await;
    client.subscribe_notifications();
    client.add_uri(vec![link],None,None,None).await?;
    Ok(())
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/aria2-leptos.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage ssr=leptos_router::SsrMode::Async/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (name,set_name) = signal("".to_string());
    let (updateValue, setUpdateValue) = signal(0);
    let links = Resource::new(  
       move || updateValue.get(),
       move |_| async move{
           fetch_links().await
       } 
    );



    view! {
        <h1>"Welcome to Leptos!"</h1>
 <Suspense
        fallback=move || view! { <p>"Loading..."</p> }
    >
        <h2>"My Data"</h2>
        {move || Suspend::new(async move {
            let links = links.get()?;
            let links = links.ok()?;
            Some(view! {
                <ul>
                    {links.iter().map(|link| view! { <li>{link.gid.clone()}  <ul>{link.files.iter().map(|x| view! { <li>{x.clone()}</li>} ).collect_view()} </ul></li> }).collect::<Vec<_>>()}
                </ul>
            })
                    })}
    </Suspense>
        
        <input type="text"
                    on:input:target=move |ev| {
                            set_name.set(ev.target().value());
                    }
                    prop:value=name
        />
        <button on:click=move |_| {
            spawn_local(async move {
                add_link(name.read().clone()).await.unwrap();
            });
            setUpdateValue.set(updateValue.get() + 1 % 3);
        }>"Add Link"</button>

    }
}
