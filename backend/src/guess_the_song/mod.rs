use std::sync::Arc;

use crate::{
    AppState, generate_lobby_code,
    state::{Lobby, ServerEvent},
};
use axum::{
    Json,
    extract::{
        State,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, stream::StreamExt};
use serde::Deserialize;

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}

#[derive(Deserialize, Debug)]
struct UserEvent {
    lobby_code: String,
    event: String,
    user_id: String,
    username: String,
    content: Option<String>,
}

struct GuessTheSongConnectionGuard {
    lobby: Arc<Lobby>,
    player_id: String,
}

impl Drop for GuessTheSongConnectionGuard {
    fn drop(&mut self) {
        println!("Players {:?}", self.lobby.get_players());
        self.lobby.player_leave(&self.player_id);
        println!(
            "Player {} disconnected and removed from lobby",
            self.player_id
        );
        println!("Players after{:?}", self.lobby.get_players());
        let _ = self.lobby.broadcast.send(ServerEvent::PlayerLeave {
            player_id: self.player_id.clone(),
        });
    }
}

pub async fn guess_the_song_create_lobby(State(state): State<AppState>) -> impl IntoResponse {
    let mut lobby_code;
    loop {
        lobby_code = generate_lobby_code();
        match state.games.get_lobby(&lobby_code) {
            Some(_) => continue,
            None => break,
        }
    }
    state.games.add_lobby(&lobby_code);
    println!("Added lobby {lobby_code}");

    Json(CreateLobbyResponse {
        lobby_code: lobby_code,
    })
}

pub async fn handle_guess_the_song(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Await the Handshake Join Request
    let join_req = match receiver.next().await {
        Some(Ok(Message::Text(m))) => m,
        _ => return,
    };
    let join_req = match serde_json::from_str::<UserEvent>(&join_req) {
        Ok(j) if j.event == "join" => j,
        _ => {
            let _ = sender
                .send(Message::Text("Expected join request".into()))
                .await;
            return;
        }
    };
    let player_id = join_req.user_id;
    let player_username = join_req.username;
    let lobby = match state.games.get_lobby(&join_req.lobby_code) {
        Some(l) => l,
        None => {
            let _ = sender.send(Message::Text("Lobby Not Found".into())).await;
            return;
        }
    };

    let _ = lobby.player_join(player_id.clone(), player_username.clone());
    let _guard = GuessTheSongConnectionGuard {
        lobby: lobby.clone(),
        player_id: player_id.clone(),
    };
    // Clone the broadcast channel into tx (Sender)
    let tx = lobby.broadcast.clone();
    // Subscribe to the broadcast channel to aquire a (Receiver)
    let mut rx = tx.subscribe();

    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&ServerEvent::SyncState {
                players: lobby.get_players(),
            })
            .expect("Failed to parse SyncState event")
            .into(),
        ))
        .await;

    // Create the send_task
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    println!("Relaying broadcast {:?}", json);
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Serialization error: {:?}", e);
                    continue;
                }
            }
        }
    });

    let _ = lobby.broadcast.send(ServerEvent::PlayerJoin {
        player_id: player_id.clone(),
        player_username: player_username.clone(),
    });

    //  Create the recv_task
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(req) => {
                    let req: UserEvent = match serde_json::from_str(&req) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed to parse user event: {:?}", e);
                            continue;
                        }
                    };
                    println!("{:?}", req);
                    match req.event.as_str() {
                        "ready" => {
                            lobby.player_ready(&player_id);
                            let _ = lobby.broadcast.send(ServerEvent::PlayerReady {
                                player_id: player_id.clone(),
                            });
                            if lobby.all_ready() {
                                let _ = lobby.broadcast.send(ServerEvent::AllReady);
                            }
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    println!("Websocket disconnected");
}
