use std::{time::Duration};

use axum::extract::ws::WebSocket;

struct Player {
    name: String,
    score: u8,
}
struct GameState {
    players: Vec<Player>,
    duration: Duration,
    input_delay: Duration,
    songs: Vec<String>,
}

pub async fn handle_guess_the_song(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            println!("Client disconnected from lobby");
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            println!("Client disconnected from lobby");
            return;
        }
    }
}