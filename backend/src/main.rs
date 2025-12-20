use crate::{guess_the_song::guess_the_song_create_lobby, state::AppState};
use axum::http::{Method, StatusCode};
use axum::{
    Router,
    extract::{Path, State, ws::WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{any, post},
};
use rand::{Rng, distr::Alphanumeric};
use tower_http::cors::{Any, CorsLayer};

mod guess_the_song;
mod spotify;
mod state;

#[tokio::main]
async fn main() {
    let s = spotify::get_spotify_client().await;
    let state = AppState::new(s);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route(
            "/guess-the-song/create-lobby",
            post(guess_the_song_create_lobby),
        )
        .route("/ws/{game}", any(handle_ws))
        .layer(cors)
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
