use std::env;

use crate::{guess_the_song::guess_the_song_create_lobby, state::AppState};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::{
    Router,
    extract::{Path, State, ws::WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{any, post},
};
use rand::{Rng, distr::Alphanumeric};
use tokio::sync::mpsc;
use tower_http::cors::{CorsLayer};

mod guess_the_song;
mod state;

#[tokio::main]
async fn main() {
    let s = guess_the_song::api::get_spotify_client().await;
    let (clean_tx, mut clean_rx) = mpsc::unbounded_channel();
    let state = AppState::new(s, clean_tx);
    let cleanup_state = state.clone();

    // Spawn cleanup thread
    tokio::spawn(async move {
        while let Some(lobby_code) = clean_rx.recv().await {
            println!("Cleaning up lobby: {}", lobby_code);
            cleanup_state.games.remove_lobby(&lobby_code);
        }
    });
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(env::var("FRONTEND_URL").expect("env.FRONTEND_URL not set").parse::<HeaderValue>().unwrap());

    let app = Router::new()
        .route(
            "/guess-the-song/create-lobby",
            post(guess_the_song_create_lobby),
        )
        .route("/ws/{game}", any(handle_ws))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", env::var("PORT").expect("env.PORT not set"))).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_ws(
    ws: WebSocketUpgrade,
    Path(game): Path<String>,
    State(state): State<AppState>,
) -> Response {
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
