use axum::{Json, extract::{State, ws::{Message, WebSocket}}, response::{IntoResponse}};
use serde::Deserialize;
use tokio::{sync::broadcast};
use futures_util::{SinkExt, stream::{StreamExt}};
use crate::{AppState, generate_lobby_code, state::Player};

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}

#[derive(Deserialize, Debug)]
struct UserEvent {
    lobby_code: String,
    action: String,
    user_id: String,
    username: String,
    content: Option<String>,
}

pub async fn guess_the_song_create_lobby(State(state): State<AppState>) -> impl IntoResponse {
    let mut lobby_code;
    loop {
        lobby_code = generate_lobby_code();
        match state.games.get_lobby(&lobby_code) {
            Some(l) => continue,
            None => break,
        }
    }

    let (send, _) = broadcast::channel::<String>(64);
    // Initialize a new lobby
    state.games.add_lobby(&lobby_code, send);

    Json(CreateLobbyResponse {
        lobby_code: lobby_code
    })
}

pub async fn handle_guess_the_song(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Await the Handshake Join Request
    let join_req = match receiver.next().await {
        Some(Ok(Message::Text(m))) => m,
        _ => return,
    };
    let join_req= match serde_json::from_str::<UserEvent>(&join_req) {
        Ok(j) if j.action == "join" => j,
        _ => {
            let _ = sender
                .send(Message::Text("Expected join request".into()))
                .await;
            return;
        }
    };
    let lobby = match state.games.get_lobby(&join_req.lobby_code) {
        Some(l) => l,
        None => {
            let _ = sender.send(Message::Text("Lobby Not Found".into())).await;
            return;
        },
    };

    // Add player to lobby
    lobby.player_join(Player {
        user_id: join_req.user_id,
        username: join_req.username,
    });

    // Clone the broadcast channel into tx (Sender)
    let tx = lobby.broadcast.clone();
    // Subscribe to the broadcast channel to aquire a (Receiver)
    let mut rx = tx.subscribe();

    /*
        Create the send_task
        Receives broadcasts and sends them to the client

        Requires
            - Rx (Broadcast Receiver)
            - Sender (Socket Sender)
     */
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    /*
        Create the recv_task, which is the following code below
        Takes in messages from the client and broadcasts them

        Requires
            - Tx (Broadcast Sender)
            - Receiver (Socket Receiver)
     */
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
                    match req.action.as_str() {
                        "ready" => {

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

    // Clean up on client disconnect
    println!("Websocket disconnected");
    // TODO remove client from lobby
    // Figure out how to remove the lobby if all clients are gone
}