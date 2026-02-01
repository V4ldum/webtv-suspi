use std::collections::HashMap;

#[cfg(feature = "ssr")]
use cached::proc_macro::cached;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
#[cfg(feature = "ssr")]
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use singlestage::{Avatar, AvatarImage, Badge, Theme, ThemeProvider};
#[cfg(feature = "ssr")]
use tokio::time::Duration;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
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

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/webtv.css" />

        <Title text="WebTV Suspicion" />

        <ThemeProvider mode="dark" theme=Theme::Default>
            <Router>
                <main class="max-w-7xl mx-auto py-4">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=path!("/") view=HomePage />
                    </Routes>
                </main>
            </Router>
        </ThemeProvider>
    }
}

#[cfg(feature = "ssr")]
struct FetchStreamer(String, String);

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct TwitchUsersResponse {
    data: Vec<StreamerUserData>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize, Clone)]
struct StreamerUserData {
    login: String,
    profile_image_url: String,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct TwitchStreamsResponse {
    data: Vec<StreamerStreamData>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize, Clone)]
struct StreamerStreamData {
    user_login: String,
    title: String,
    viewer_count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Streamer {
    display_name: String,
    channel_name: String,
    avatar_url: String,
    is_live: bool,
    viewer_count: Option<u32>,
    stream_title: Option<String>,
}

impl Streamer {
    fn from(fetch: FetchStreamer, user: StreamerUserData, stream: Option<StreamerStreamData>) -> Self {
        Self {
            display_name: fetch.0,
            channel_name: fetch.1,
            avatar_url: user.profile_image_url,
            is_live: stream.is_some(),
            viewer_count: stream.as_ref().map(|s| s.viewer_count),
            stream_title: stream.map(|s| s.title),
        }
    }
}

#[cfg(feature = "ssr")]
#[derive(Deserialize)]
struct ClientCredentials {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

#[cfg(feature = "ssr")]
#[cached(
    time = 36000,
    result = true,
    sync_writes = "default",
    key = "String",
    convert = r#"{ "users".to_string() }"#
)]
async fn fetch_users_data(
    client: &Client,
    streamers_to_fetch: &[FetchStreamer],
) -> Result<HashMap<String, StreamerUserData>, ServerFnError> {
    let request_params = streamers_to_fetch
        .iter()
        .map(|s| format!("login={}", s.1))
        .collect::<Vec<_>>()
        .join("&");

    let users_response = client
        .get(format!("https://api.twitch.tv/helix/users?{}", request_params))
        .send()
        .await?
        .json::<TwitchUsersResponse>()
        .await?;

    Ok(users_response
        .data
        .into_iter()
        .map(|u| (u.login.clone().to_lowercase(), u))
        .collect::<HashMap<_, _>>())
}

#[cfg(feature = "ssr")]
#[cached(
    time = 300,
    result = true,
    sync_writes = "default",
    key = "String",
    convert = r#"{ "streams".to_string() }"#
)]
async fn fetch_streams_data(
    client: &Client,
    streamers_to_fetch: &[FetchStreamer],
) -> Result<HashMap<String, StreamerStreamData>, ServerFnError> {
    let request_params = streamers_to_fetch
        .iter()
        .map(|s| format!("user_login={}", s.1))
        .collect::<Vec<_>>()
        .join("&");

    let streams_response = client
        .get(format!("https://api.twitch.tv/helix/streams?{}", request_params))
        .send()
        .await?
        .json::<TwitchStreamsResponse>()
        .await?;

    Ok(streams_response
        .data
        .into_iter()
        .map(|s| (s.user_login.clone(), s))
        .collect::<HashMap<_, _>>())
}

#[server(GetStreamers)]
async fn fetch_streamers() -> Result<Vec<Streamer>, ServerFnError> {
    use axum::http::{HeaderMap, HeaderValue};

    // Secrets
    let client_id = dotenvy::var("TWITCH_CLIENT_ID").map_err(|_| ServerFnError::new("Missing TWITCH_CLIENT_ID"))?;
    let client_secret =
        dotenvy::var("TWITCH_CLIENT_SECRET").map_err(|_| ServerFnError::new("Missing TWITCH_CLIENT_SECRET"))?;
    let client_credentials = ClientCredentials {
        access_token: "2ajw960ob4vvou7n2rc0n9zo9dnt9z".to_string(),
        expires_in: 5495521,
        token_type: "bearer".to_string(),
    };

    // Streamers to fetch
    let streamers_to_fetch = vec![
        FetchStreamer("Shokk".to_string(), "shokkfamedslayer".to_string()),
        FetchStreamer("Cuzdot".to_string(), "cuzdot".to_string()),
        FetchStreamer("Eden".to_string(), "edenwod".to_string()),
        FetchStreamer("Taco".to_string(), "tacokek".to_string()),
        FetchStreamer("TT".to_string(), "t_t_27".to_string()),
        FetchStreamer("Turbo".to_string(), "Turbogronil".to_string()),
        FetchStreamer("Anda".to_string(), "Andazara".to_string()),
        FetchStreamer("Tinky".to_string(), "tinky_lol".to_string()),
        FetchStreamer("Vaelin".to_string(), "vaelinhc".to_string()),
        FetchStreamer("Dife".to_string(), "zilakin".to_string()),
        FetchStreamer("Cruzz Croix V".to_string(), "cruzzxv".to_string()),
        FetchStreamer("Spanra".to_string(), "spannra".to_string()),
    ];

    // Query Twitch
    let mut headers = HeaderMap::new();
    headers.insert("Client-ID", HeaderValue::from_str(&client_id)?);
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", client_credentials.access_token))?,
    );
    let client = reqwest::Client::builder().default_headers(headers).build()?;

    let res = tokio::try_join!(
        fetch_users_data(&client, &streamers_to_fetch),
        fetch_streams_data(&client, &streamers_to_fetch)
    );
    let (mut users_map, mut streams_map) = res?;

    let ret = streamers_to_fetch
        .into_iter()
        .filter_map(|s| {
            let user = users_map.remove(&s.1.to_lowercase())?;
            let stream = streams_map.remove(&s.1.to_lowercase());

            Some(Streamer::from(s, user, stream))
        })
        .collect();

    Ok(ret)
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let streamers = Resource::new(|| (), |_| fetch_streamers());

    view! {
        <div class="px-4">
            <iframe
                src="https://twitch.tv/embed/yarrgi?parent=127.0.0.1"
                class="w-full aspect-video"
                // height="425"
                // width="720"
                // frameborder="0"
                // scrolling="no"
                allowfullscreen="true"
            ></iframe>

            <div class="my-12">
                <h2 class="text-xl font-bold">"Roster"</h2>
                <Suspense fallback=move || {
                    view! { <p>"Loading streamers..."</p> }
                }>
                    {move || {
                        streamers
                            .get()
                            .map(|result| {
                                result
                                    .map(|streamers| {
                                        view! {
                                            <div class="grid grid-cols-4 gap-x-4 gap-y-8 w-full my-4">
                                                {streamers
                                                    .into_iter()
                                                    .map(|streamer| {
                                                        view! {
                                                            <div
                                                                class="w-72 rounded-lg hover:bg-accent/40"
                                                                on:click=move |_| {
                                                                    leptos::logging::log!("Clicked on streamer");
                                                                }
                                                            >
                                                                <div class="relative aspect-video">
                                                                    // Stream preview
                                                                    <img
                                                                        src=format!(
                                                                            "https://static-cdn.jtvnw.net/previews-ttv/live_user_{}-854x480.jpg",
                                                                            streamer.channel_name,
                                                                        )
                                                                        alt="Stream Preview"
                                                                        class="rounded-lg"
                                                                    />
                                                                    // Stream status
                                                                    {if streamer.is_live {
                                                                        view! {
                                                                            <Badge
                                                                                class="absolute top-2 left-2 bg-red-600/60"
                                                                                variant="destructive"
                                                                            >
                                                                                "LIVE"
                                                                            </Badge>
                                                                        }
                                                                    } else {
                                                                        view! {
                                                                            <Badge
                                                                                class="absolute top-2 left-2 bg-secondary/80"
                                                                                variant="secondary"
                                                                            >
                                                                                "OFFLINE"
                                                                            </Badge>
                                                                        }
                                                                    }}
                                                                    // Viewer Count
                                                                    {streamer
                                                                        .viewer_count
                                                                        .map(|viewer_count| {
                                                                            view! {
                                                                                <Badge
                                                                                    class="absolute bottom-2 right-2 bg-secondary/80"
                                                                                    variant="secondary"
                                                                                >

                                                                                    {viewer_count}
                                                                                </Badge>
                                                                            }
                                                                        })}
                                                                </div>
                                                                // Streamer avatar
                                                                <div class="flex flex-row items-center mx-2 my-3">
                                                                    <Avatar class="mr-2 w-9 h-9">
                                                                        <AvatarImage
                                                                            src=streamer.avatar_url
                                                                            alt=streamer.channel_name.as_ref()
                                                                            class="rounded-full w-full h-full object-cover"
                                                                        />
                                                                    </Avatar>
                                                                    // Stream title
                                                                    <div class="flex flex-col">
                                                                        <p class="text-md font-semibold line-clamp-1">
                                                                            {streamer.stream_title}
                                                                        </p>
                                                                        // Streamer display name
                                                                        <p class="text-sm text-muted-foreground">
                                                                            {streamer.display_name}
                                                                        </p>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </div>
                                        }
                                    })
                                    .ok()
                            })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
