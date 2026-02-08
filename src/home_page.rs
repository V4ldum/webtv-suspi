use leptos::prelude::*;
use singlestage::{Avatar, AvatarImage, Badge};

use crate::fetch_streamers::{Streamer, fetch_streamers};

#[component]
fn StreamerCard(streamer: Streamer, featured: RwSignal<Option<String>>) -> impl IntoView {
    let is_live_featured = streamer.is_live;
    let channel_name_featured = streamer.channel_name.to_lowercase();

    view! {
        <div
            class="w-72 rounded-lg hover:bg-accent/40"
            on:click=move |_| {
                if is_live_featured && featured.get().as_ref() != Some(&channel_name_featured) {
                    featured.set(Some(channel_name_featured.clone()));
                }
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
                                " viewers"
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
}

#[component]
pub fn HomePage() -> impl IntoView {
    let streamer_response = Resource::new(|| (), |_| fetch_streamers());
    let featured = RwSignal::new(None);
    Effect::new(move || {
        if let Some(Ok(streamer_response)) = streamer_response.get()
            && let Some(first_streamer) = streamer_response.streamers.first()
            && first_streamer.is_live
        {
            featured.set(Some(first_streamer.channel_name.to_lowercase()))
        }
    });

    view! {
        <div class="px-4">
            // Stream
            <div class="w-full aspect-video">
                <Suspense fallback=move || {
                    view! { <p>"Loading..."</p> }
                }>
                    {move || {
                        let data = streamer_response.get();
                        match data {
                            Some(Ok(response)) => {
                                match featured.get() {
                                    Some(channel_name) => {

                                        view! {
                                            <iframe
                                                src=format!(
                                                    "https://player.twitch.tv/?channel={channel_name}&parent={}",
                                                    response.base_addr,
                                                )
                                                class="w-full h-full"
                                                // height="425"
                                                // width="720"
                                                // frameborder="0"
                                                // scrolling="no"
                                                allowfullscreen="true"
                                            ></iframe>
                                        }
                                            .into_any()
                                    }
                                    None => {
                                        view! {
                                            <div class="flex items-center justify-center w-full h-full">
                                                <p class="text-xl font-semibold">
                                                    "Frérot y'a personne qui stream"
                                                </p>
                                            </div>
                                        }
                                            .into_any()
                                    }
                                }
                            }
                            _ => {

                                view! {
                                    <div class="flex items-center justify-center w-full h-full">
                                        <p class="text-xl font-semibold">"CKC"</p>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
            // Roster
            <div class="my-12">
                <h2 class="text-xl font-bold">"Roster"</h2>
                <Suspense fallback=move || {
                    view! { <p>"Loading streamers..."</p> }
                }>
                    {move || {
                        streamer_response
                            .get()
                            .map(|result| {
                                result
                                    .map(|streamer_response| {
                                        view! {
                                            <div class="grid grid-cols-4 gap-x-4 gap-y-8 w-full my-4">
                                                {streamer_response
                                                    .streamers
                                                    .into_iter()
                                                    .map(|streamer| {
                                                        view! {
                                                            <StreamerCard streamer featured />
                                                        }
                                                    } )
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
