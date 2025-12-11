use axum::{
    Json, Router, extract::{Path, ws::{WebSocketUpgrade}}, response::{IntoResponse, Response}, routing::{any, get, post}
};
use tower_http::cors::{Any, CorsLayer};
use axum::http::{Method, StatusCode};
use rand:: {
    Rng,
    distr::Alphanumeric
};

mod guess_the_song;

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}

#[tokio::main]
async fn main() {

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);


    let app = Router::new()
        .route("/guess-the-song/create-lobby", post(create_lobby))
        .route("/ws/{game}", any(handle_ws))
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_ws(ws: WebSocketUpgrade, Path(game): Path<String>) -> Response {
    match game.as_str() {
        "guess-the-song" => { ws.on_upgrade(move |socket| guess_the_song::handle_guess_the_song(socket)) },
        _ => (StatusCode::NOT_FOUND, "Game mode not found").into_response()
    }
}


async fn create_lobby() -> impl IntoResponse {
    let lobby_code = generate_lobby_code();
    Json(CreateLobbyResponse {
        lobby_code: lobby_code
    })
}

fn generate_lobby_code() -> String {
    rand::rng().sample_iter(&Alphanumeric).take(6).map(char::from).collect()
}