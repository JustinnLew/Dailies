use std::env;

// src/spotify.rs
use rspotify::{ClientCredsSpotify, Credentials};
use dotenv::dotenv;

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