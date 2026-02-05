#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use std::sync::OnceLock;
use std::time::Duration;
#[cfg(feature = "ssr")]
use tokio::sync::Mutex;

#[cfg(feature = "ssr")]
struct CachedCredentials {
    credentials: CredentialsResponse,
    expiration_date: DateTime<Utc>,
}

#[cfg(feature = "ssr")]
static CREDENTIALS: OnceLock<Mutex<Option<CachedCredentials>>> = OnceLock::new();

#[cfg(feature = "ssr")]
#[derive(Deserialize)]
struct CredentialsResponse {
    access_token: String,
    expires_in: u64,
    //token_type: String,
}

#[cfg(feature = "ssr")]
#[derive(Serialize)]
struct CredentialsForm {
    client_id: String,
    client_secret: String,
    grant_type: String,
}

#[cfg(feature = "ssr")]
pub async fn get_access_token() -> Result<String, ServerFnError> {
    use chrono::Days;

    let now = Utc::now();

    let lock = CREDENTIALS.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().await;

    if let Some(cached_credentials) = guard.as_ref()
        && cached_credentials
            .expiration_date
            .checked_sub_days(Days::new(1))
            .expect("No issue with UTC date")
            > now
    {
        // Access token found and not expired/close to expire
        Ok(cached_credentials.credentials.access_token.clone())
    } else {
        // No access token found or access token expired/close to expire

        let client_id = dotenvy::var("TWITCH_CLIENT_ID").map_err(|_| ServerFnError::new("Missing TWITCH_CLIENT_ID"))?;
        let client_secret =
            dotenvy::var("TWITCH_CLIENT_SECRET").map_err(|_| ServerFnError::new("Missing TWITCH_CLIENT_SECRET"))?;

        // Query new token
        let credentials = reqwest::Client::new()
            .post("https://id.twitch.tv/oauth2/token")
            .form(&CredentialsForm {
                client_id,
                client_secret,
                grant_type: "client_credentials".to_string(),
            })
            .send()
            .await?
            .json::<CredentialsResponse>()
            .await?;

        // Set credentials in cache
        let expiration_date = Utc::now() + Duration::from_secs(credentials.expires_in);
        let cached_credentials = CachedCredentials {
            credentials,
            expiration_date,
        };
        let access_token = cached_credentials.credentials.access_token.clone();

        *guard = Some(cached_credentials);

        Ok(access_token)
    }
}
