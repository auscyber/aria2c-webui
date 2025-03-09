use std::sync::Arc ;

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
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct TorrentInfo {
    pub name: String
}

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct TorrentData {
    comment: Option<String>,
    info: Option<TorrentInfo>
}

use serde_with::{serde_as, DisplayFromStr};


#[serde_as]
#[derive(Serialize, Deserialize, Clone,Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinkData {
    gid: String,
    status: String,
    #[serde_as(as="DisplayFromStr")]
    total_length: u64,
    #[serde_as(as="DisplayFromStr")]
    completed_length: u64,
    #[serde_as(as="DisplayFromStr")]
    download_speed: u64,
   #[serde_as(as="DisplayFromStr")]
    upload_speed: u64,
    #[serde_as(as="DisplayFromStr")]
    connections: u64,
    bittorrent: Option<TorrentData>,
    #[serde_as(as="Option<DisplayFromStr>")]
    num_seeders: Option<u64>,
    #[serde_as(as="DisplayFromStr")]
    num_pieces: u64,
    #[serde_as(as="DisplayFromStr")]
    piece_length: u64,
    dir: String,
//    files: Vec<String>,
}

#[server]
async fn fetch_links() -> Result<Vec<LinkData>, server_fn::error::ServerFnError> {
    use crate::AppState;
    use tokio::sync::RwLock;
    let state = expect_context::<Arc<RwLock<aria2_ws::Client>>>();
    let client = state.read().await;
    let b: Vec<LinkData>=  client.call_and_wait("tellActive", Vec::new()).await?;
    let c: Vec<LinkData> = client.call_and_wait("tellStopped", vec![serde_json::Value::from(0), serde_json::Value::from(1000) ]).await?;
    let d: Vec<LinkData> = client.call_and_wait("tellWaiting", vec![serde_json::Value::from(0), serde_json::Value::from(1000) ]).await?;
    let b = b.iter().chain(c.iter()).chain(d.iter()).map(|x| x.clone()).collect::<Vec<_>>();
    Ok(b)
}

#[server]
async fn add_link(link: String) -> Result<(), server_fn::error::ServerFnError> {
    use crate::AppState;
    use tokio::sync::RwLock;
    let state = expect_context::<Arc<RwLock<aria2_ws::Client>>>();
    let client = state.read().await;
    client.add_uri(vec![link], None, None, None).await?;
    Ok(())
}

#[server]
async fn add_torrent(torrent: String) -> Result<(), server_fn::error::ServerFnError> {
    use crate::AppState;
    use tokio::sync::RwLock;
    let state = expect_context::<Arc<RwLock<aria2_ws::Client>>>();
    let client = state.read().await;
    client.add_torrent(&torrent, None, None, None, None).await?;
    Ok(())
}





#[component]
pub fn App() -> impl IntoView {
        leptos_ws::provide_websocket("ws://localhost:3000/ws");
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/aria2-leptos.css" />

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage ssr=leptos_router::SsrMode::Async />
                </Routes>
            </main>
        </Router>
    }
}
#[cfg(feature = "ssr")]
use aria2_ws::Notification;

#[derive(Debug, Clone, PartialEq, Eq,Deserialize, Serialize)]
struct NotificationsState {
    notifications: Vec<String>,
}

#[server]
async fn delete_link(gid: String) -> Result<(), server_fn::error::ServerFnError> {
    use crate::AppState;
    let state = expect_context::<AppState>();
    let client = state.aria2.read().await;
    client.remove_download_result(&gid).await.ok().map(|_| ());
    client.force_remove(&gid).await?;
    Ok(())
}


#[server]
async fn poll_notifications() -> Result<(), server_fn::error::ServerFnError> {
    use crate::AppState;
    use tokio::sync::RwLock;
    let state = expect_context::<Arc<RwLock<aria2_ws::Client>>>();
    let client = state.read().await;
    let mut channel = client.subscribe_notifications();
    let server_signal = leptos_ws::ServerSignal::new("aria2_notifications".to_string(), NotificationsState {
        notifications: vec![],
    }).unwrap();

    while let Ok(msg) = channel.recv().await {
        match msg {
            Notification::Aria2 { gid, .. } => {
                let status = client.tell_status(&gid).await?;
                println!("Received notification {:?}", status);
        server_signal.update(move |state| {
            state.notifications.push(gid.clone());
        });
        
                server_signal.notify();
            },
            _ => {}
        }
        
    }
    
    Ok(())
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
//    window().navigator().register_protocol_handler(scheme, url, title)
    let (name, set_name) = signal("".to_string());
    let _ = LocalResource::new(move || async {
            poll_notifications().await.unwrap();
    });
    let notifications = leptos_ws::ServerSignal::new("aria2_notifications".to_string(), NotificationsState {
        notifications: vec![],
    }).unwrap();
    let links = LocalResource::new(
        move ||  {
            let notifications = notifications.clone();
        async move { 
            let _ = notifications.track();
            let links = fetch_links().await?;
       println!("Links reload: {:?}", links); 
        Ok::<_,server_fn::ServerFnError>(links) 
    }
}
    );

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <h2>"My Data"</h2>
            {move || Suspend::new(async move {
                let links = links.get()?;
                let links = links.as_ref().ok()?;
                println!("Links: {:?}", links);
                Some(
                    view! {
                        <div class="bg-gray-100">
                            <button on:click=move |_| {
                                leptos::logging::log!("WOWW")
                            }>Test </button>
                            {links
                                .into_iter()
                                .map(|link| {
                                    let gid = link.gid.clone();
                                    view! {
                                        <div class="flex flex-row gap-4 w-full">
                                        {link.status.clone()}
                                            <button
                                                class="bg-blue-400 rounded-md p-2"
                                                on:click=move |_| {
                                                    leptos::logging::log!("Deleting link");
                                                    let gid = gid.clone();
                                                    spawn_local(async move {
                                                        match delete_link(gid).await {
                                                            Ok(_) => {
                                                                leptos::logging::log!("Deleted link");
                                                            }
                                                            Err(e) => {
                                                                leptos::logging::log!("Error deleting link: {:?}", e);
                                                        }
                                                    }
                                                    });
                                                }
                                            >
                                                "Delete Link"
                                            </button>
                                            {link.gid.clone()}
                                            {link.bittorrent.as_ref().and_then(|x| x.info.as_ref().map(|x| x.name.clone())).unwrap_or("".to_string())}
                                            <ul>
                                                                                            </ul>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    },
                )
            })}
        </Suspense>

        <input
            type="text"
            on:input:target=move |ev| {
                set_name.set(ev.target().value());
            }
            prop:value=name
        />
        <button on:click=move |_| {
            spawn_local(async move {
                add_link(name.read().clone()).await.unwrap();
            });
        }>"Add Link"</button>

        
    }
}
