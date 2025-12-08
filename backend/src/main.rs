use axum::{
    Json, Router, extract::{Path, ws::{WebSocket, WebSocketUpgrade}}, response::{IntoResponse, Response}, routing::{any, get}
};
use tower_http::cors::{Any, CorsLayer};
use axum::http::{Method, StatusCode};
use rand:: {
    Rng,
    distr::Alphanumeric
};

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


    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/guess-the-song/create-lobby", get(create_lobby))
        .route("/ws/{game}/{lobby_code}", any(handle_ws))
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_ws(ws: WebSocketUpgrade, Path((game, lobby_code)): Path<(String, String)>) -> Response {
    match game.as_str() {
        "guess-the-song" => { ws.on_upgrade(move |socket| handle_guess_the_song(socket, lobby_code)) },
        _ => (StatusCode::NOT_FOUND, "Game mode not found").into_response()
    }
}

async fn handle_guess_the_song(mut socket: WebSocket, lobby_code: String) {
    println!("Client connected to lobby {}", lobby_code);

     while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            println!("Client disconnected from lobby {}", lobby_code);
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            println!("Client disconnected from lobby {}", lobby_code);
            return;
        }
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