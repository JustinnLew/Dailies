use std::env;
use std::time::Duration;

use crate::{guess_the_song::guess_the_song_create_lobby, state::AppState};
use axum::http::StatusCode;
use axum::{
    Router,
    extract::{Path, State, ws::WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{any, post},
};
use rand::{Rng, distr::Alphanumeric};
use tokio::sync::mpsc;
use tokio::time::interval;
use tower_http::cors::{CorsLayer};
use tracing::{Instrument, Level, info, instrument};

mod guess_the_song;
mod state;

#[tokio::main]
async fn main() {
    // Setup logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::INFO)
        .init();
    info!("Starting server...");

    let s = guess_the_song::api::get_spotify_client().await;
    let (clean_tx, mut clean_rx) = mpsc::unbounded_channel();
    let state = AppState::new(s, clean_tx);
    let cleanup_state = state.clone();
    let scan_state = state.clone();

    // Spawn direct cleanup thread
    tokio::spawn(
        async move {
            while let Some(lobby_code) = clean_rx.recv().await {
                info!("Cleaning up lobby: {}", lobby_code);
                cleanup_state.games.remove_lobby(&lobby_code);
            }
        }
        .instrument(tracing::info_span!("CLEANUP")),
    );
    // Spawn periodic cleanup
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            // WILL NEED TO BE CHANGED FOR FUTURE GAMES
            // PROBABLY USE THE REGISTRY TO MAP FROM ID TO GAME THEN RETAIN
            scan_state
                .games
                .guess_the_song
                .retain(|_, v| !v.lobby_state.lock().unwrap().empty());
        }
    });

    let app = Router::new()
        .route(
            "/api/guess-the-song/create-lobby",
            post(guess_the_song_create_lobby),
        )
        .route("/api/{game}", any(handle_ws))
        .layer(CorsLayer::very_permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or("3000".to_string())
    ))
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[instrument(skip(ws, state))]
async fn handle_ws(
    ws: WebSocketUpgrade,
    Path(game): Path<String>,
    State(state): State<AppState>,
) -> Response {
    info!("Websocket Connecting");
    match game.as_str() {
        "guess-the-song" => {
            ws.on_upgrade(move |socket| guess_the_song::handle_guess_the_song(socket, state))
        }
        _ => (StatusCode::NOT_FOUND, "Game mode not found").into_response(),
    }
}

fn generate_lobby_code() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}
