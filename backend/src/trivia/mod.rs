use axum::{
    Json,
    extract::{
        State,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tracing::{Instrument, info, instrument, warn};

use crate::{
    AppState,
    connections::ConnectionGuard,
    generate_lobby_code,
    state::{LobbyServerEvent, TriviaClientEvent, TriviaGame, TriviaGameEvent, TriviaServerEvent},
};

pub mod api;

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}

#[instrument(name = "CREATE TRIVIA LOBBY", skip(state))]
pub async fn trivia_create_lobby(State(state): State<AppState>) -> impl IntoResponse {
    let mut lobby_code;
    loop {
        lobby_code = generate_lobby_code();
        if !state.games.valid_lobby_code(&lobby_code) {
            break;
        }
    }
    state.games.add_trivia_lobby(&lobby_code);
    info!("Added trivia lobby {lobby_code}");

    Json(CreateLobbyResponse { lobby_code })
}

pub async fn handle_trivia(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Use abstracted await_join_req to perform the join handshake
    let (lobby_code, player_username) =
        match TriviaGame::await_join_req(&mut receiver, &mut sender).await {
            Ok((lobby_code, player_username)) => (lobby_code, player_username),
            Err(_) => return,
        };

    let game_obj = match state.games.trivia.get(&lobby_code) {
        Some(g) => g.clone(),
        None => {
            info!("Lobby not found: {}", lobby_code);
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&&TriviaServerEvent::LobbyEvent(
                        LobbyServerEvent::JoinError {
                            message: "Lobby not found".to_string(),
                        },
                    ))
                    .unwrap()
                    .into(),
                ))
                .await;
            return;
        }
    };

    let player_id = game_obj.get_new_player_id();

    let connection_span = tracing::info_span!(
        "trivia_connection",
        lobby=%lobby_code,
        player=%player_id,
    );

    match game_obj.player_join(player_id.clone(), player_username.clone()) {
        Ok(_) => {
            info!("Player: {}, joined trivia lobby: {}", player_id, lobby_code);
        }
        Err(e) => {
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&TriviaServerEvent::LobbyEvent(
                        LobbyServerEvent::JoinError {
                            message: e.to_string(),
                        },
                    ))
                    .unwrap()
                    .into(),
                ))
                .await;
            return;
        }
    }

    let _guard = ConnectionGuard {
        game: game_obj.clone(),
        player_id: player_id.clone(),
        cleanup_tx: state.cleanup.clone(),
    };

    let tx = game_obj.broadcast.clone();
    let mut rx = tx.subscribe();

    // Send initial SyncState to the newly connected player
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&TriviaServerEvent::GameEvent(TriviaGameEvent::SyncState {
                players: game_obj.get_players(),
                settings: game_obj.get_settings(),
                leaderboard: game_obj.get_leaderboard(),
                status: game_obj.get_lobby_status(),
            }))
            .expect("Failed to parse SyncState event")
            .into(),
        ))
        .await;

    // Spawn task to forward server broadcast events to client
    let mut send_task: tokio::task::JoinHandle<()> = tokio::spawn(
        async move {
            while let Ok(msg) = rx.recv().await {
                match serde_json::to_string(&msg) {
                    Ok(json) => {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Serialization error: {:?}", e);
                        continue;
                    }
                }
            }
        }
        .instrument(connection_span.clone()),
    );

    // Notify other players about player join
    let _ = game_obj.broadcast.send(TriviaServerEvent::LobbyEvent(
        LobbyServerEvent::PlayerJoin {
            player_id: player_id.clone(),
            player_username: player_username.clone(),
        },
    ));

    // Spawn task to receive events from client, mirroring GeoGuessr's split lobby/game event execution
    let mut recv_task = tokio::spawn(
        async move {
            use futures_util::StreamExt;
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(req) => {
                        let event: TriviaClientEvent = match serde_json::from_str(&req) {
                            Ok(r) => r,
                            Err(e) => {
                                warn!("Failed to parse client event: {:?}", e);
                                continue;
                            }
                        };
                        match event {
                            TriviaClientEvent::LobbyEvent(e) => {
                                game_obj.handle_lobby_event(player_id, e);
                            }
                            TriviaClientEvent::GameEvent(e) => {
                                game_obj.handle_game_event(player_id, e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        .instrument(connection_span),
    );

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    info!("Trivia websocket disconnected");
}
