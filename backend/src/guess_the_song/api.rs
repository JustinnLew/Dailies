use std::{env, sync::Arc};

// src/spotify.rs
use dotenv::dotenv;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use rspotify::{ClientCredsSpotify, Credentials, clients::BaseClient, model::PlaylistId};

use crate::state::{GuessTheSongGame, SongState};

pub async fn get_spotify_client() -> ClientCredsSpotify {
    dotenv().ok();
    let id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");
    let creds = Credentials::new(&id, &secret);
    // ClientCredsSpotify is used for the "Client Credentials Flow"
    // (Server-to-Server, no user login required)
    let spotify = ClientCredsSpotify::new(creds);

    // This requests the initial token.
    // rspotify creates a specialized client that checks token expiration
    // before every future request and refreshes it automatically if needed.
    spotify
        .request_token()
        .await
        .expect("Failed to get initial Spotify token");

    spotify
}

pub async fn load_songs(
    spotify_client: &ClientCredsSpotify,
    playlist_link: &str,
    game: Arc<GuessTheSongGame>,
) {
    // Extract playlist ID from link
    let playlist_id = playlist_link.split("playlist/").nth(1);
    let playlist_id = match playlist_id {
        Some(id) => PlaylistId::from_id(id).unwrap(),
        None => {
            println!("Failed to extract playlist ID from link");
            return;
        }
    };
    println!("Loading tracks from playlist: {:?}", playlist_id);
    let mut playlist_items = match spotify_client.playlist(playlist_id, None, None).await {
        Ok(r) => r.tracks,
        Err(e) => {
            println!("Failed to fetch playlist: {:?}", e);
            return;
        }
    };

    // Shuffle tracks
    let mut rng = {
        let mut rng = rand::rng();
        StdRng::from_rng(&mut rng)
    };
    playlist_items.items.shuffle(&mut rng);

    // First obtain isrc, then make another request to deezer endpoint to get preview URLs
    for item in playlist_items
        .items
        .iter()
        .take(game.get_num_songs() as usize)
    {
        if let Some(rspotify::model::PlayableItem::Track(track)) = &item.track {
            if let Some(isrc) = track.external_ids.get("isrc") {
                let deezer_url = format!("https://api.deezer.com/track/isrc:{}", isrc);

                // Fetch from Deezer
                if let Ok(resp) = reqwest::get(&deezer_url).await {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(preview_url) = json["preview"].as_str() {
                            println!("Success! Preview URL for {}: {}", track.name, preview_url);

                            game.state.lock().unwrap().add_song(SongState {
                                title: (track.name.clone(), false),
                                artists: track
                                    .artists
                                    .iter()
                                    .map(|a| (a.name.clone(), false))
                                    .collect(),
                                url: preview_url.to_string(),
                            });
                        } else {
                            // ISRC isn't found
                            println!("No preview found for ISRC: {}", isrc);
                        }
                    }
                }
            }
        }
    }
}
