use std::{env, sync::Arc};

// src/spotify.rs
use rspotify::{ClientCredsSpotify, Credentials, clients::BaseClient, model::{PlaylistId}};
use dotenv::dotenv;

use crate::state::{Lobby, Song};

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
    spotify.request_token().await.expect("Failed to get initial Spotify token");

    spotify
}

pub async fn load_tracks(spotify_client: &ClientCredsSpotify, lobby: Arc<Lobby>, playlist_link: &str) -> Vec<Song> {
    let mut res = vec![];

    // Extract playlist ID from link
    let playlist_id = playlist_link
        .split("playlist/")
        .nth(1);
    let playlist_id = match playlist_id {
        Some(id) => PlaylistId::from_id(id).unwrap(),
        None => {
            println!("Failed to extract playlist ID from link");
            return res;
        }
    };
    println!("Loading tracks from playlist: {:?}", playlist_id);
    let playlist_items = match spotify_client.playlist(playlist_id, None, None).await {
        Ok(r) => r.tracks,
        Err(e) => {
            println!("Failed to fetch playlist: {:?}", e);
            return res;
        }
    };
    // First obtain isrc, then make another request to deezer endpoint to get preview URLs
    for item in playlist_items.items {
        if let Some(rspotify::model::PlayableItem::Track(track)) = item.track {
            if let Some(isrc) = track.external_ids.get("isrc") {
                let deezer_url = format!("https://api.deezer.com/track/isrc:{}", isrc);
                
                // Fetch from Deezer (Handling errors locally to keep the return type Vec<Song>)
                if let Ok(resp) = reqwest::get(&deezer_url).await {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(preview_url) = json["preview"].as_str() {
                            println!("Success! Preview URL for {}: {}", track.name, preview_url);
                            
                            // 3. Push to your results vector
                            res.push(Song {
                                title: track.name.clone(),
                                artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                                url: preview_url.to_string(),
                            });
                        } else {
                            // Deezer sometimes returns an "error" object if ISRC isn't found
                            println!("No preview found for ISRC: {}", isrc);
                        }
                    }
                }
            }
        }
    }
    
    res
}