#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::set_header::SetResponseHeaderLayer;
    use webtv::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // Content Security Policy for Twitch embed
    let is_dev = leptos_options.env == leptos::config::Env::DEV;

    let connect_src = if is_dev {
        // Allow live reload WebSocket in dev
        "connect-src 'self' ws://127.0.0.1:3001 https://api.twitch.tv https://gql.twitch.tv https://*.twitch.tv wss://*.twitch.tv"
    } else {
        "connect-src 'self' https://api.twitch.tv https://gql.twitch.tv https://*.twitch.tv wss://*.twitch.tv"
    };

    let csp_value = [
        "default-src 'self'",
        "frame-src https://player.twitch.tv https://www.twitch.tv https://twitch.tv https://embed.twitch.tv",
        // unsafe-inline and unsafe-eval required by Twitch embed
        "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://embed.twitch.tv https://player.twitch.tv https://static.twitchcdn.net",
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' https://*.twitch.tv https://static-cdn.jtvnw.net data:",
        connect_src,
        "media-src 'self' https://*.twitch.tv https://*.ttvnw.net blob:",
        "worker-src 'self' blob:",
    ]
    .join("; ");

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::from_str(&csp_value).unwrap(),
        ));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
