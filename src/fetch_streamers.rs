#[cfg(feature = "ssr")]
use cached::proc_macro::cached;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[cfg(feature = "ssr")]
use std::{cmp::Reverse, collections::HashMap};

#[cfg(feature = "ssr")]
use crate::get_credentials::get_access_token;

struct FetchStreamer(String, String);

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct TwitchUsersResponse {
    data: Vec<StreamerUserData>,
}

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

#[derive(Debug, Deserialize, Clone)]
struct StreamerStreamData {
    user_login: String,
    title: String,
    viewer_count: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Streamer {
    pub display_name: String,
    pub channel_name: String,
    pub avatar_url: String,
    pub is_live: bool,
    pub viewer_count: Option<u32>,
    pub stream_title: Option<String>,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamerResponse {
    pub base_addr: String,
    pub streamers: Vec<Streamer>,
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
pub async fn fetch_streamers() -> Result<StreamerResponse, ServerFnError> {
    use axum::http::{HeaderMap, HeaderValue};

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

    // Credentials
    let client_id = dotenvy::var("TWITCH_CLIENT_ID").map_err(|_| ServerFnError::new("Missing TWITCH_CLIENT_ID"))?;
    let access_token = get_access_token().await?;

    // Query Twitch
    let mut headers = HeaderMap::new();
    headers.insert("Client-ID", HeaderValue::from_str(&client_id)?);
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", &access_token))?,
    );
    let client = reqwest::Client::builder().default_headers(headers).build()?;

    let res = tokio::try_join!(
        fetch_users_data(&client, &streamers_to_fetch),
        fetch_streams_data(&client, &streamers_to_fetch)
    );
    let (mut users_map, mut streams_map) = res?;

    let mut streamers = streamers_to_fetch
        .into_iter()
        .filter_map(|s| {
            let user = users_map.remove(&s.1.to_lowercase())?;
            let stream = streams_map.remove(&s.1.to_lowercase());

            Some(Streamer::from(s, user, stream))
        })
        .collect::<Vec<_>>();

    let base_addr = dotenvy::var("BASE_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());

    streamers.sort_by_key(|s| {
        (
            Reverse(s.is_live),
            Reverse(s.viewer_count.unwrap_or(0)),
            s.display_name.to_lowercase(),
        )
    });
    Ok(StreamerResponse { base_addr, streamers })
}
