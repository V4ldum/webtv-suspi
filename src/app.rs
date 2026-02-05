use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use singlestage::{Theme, ThemeProvider};

use crate::home_page::HomePage;

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
