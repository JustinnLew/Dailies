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
pub mod api;

use crate::{
    connections::ConnectionGuard,
    generate_lobby_code,
    state::{
        AppState, LobbyServerEvent,
        geoguessr::{GeoGuesserClientEvent, GeoGuessr, GeoGuessrGameEvent, GeoGuessrServerEvent},
    },
};

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}

#[instrument(name = "CREATE LOBBY", skip(state))]
pub async fn create_geo_guessr_lobby(State(state): State<AppState>) -> impl IntoResponse {
    let mut lobby_code;
    loop {
        lobby_code = generate_lobby_code();
        if !state.games.valid_lobby_code(&lobby_code) {
            break;
        }
    }
    state.games.add_geo_guessr_lobby(&lobby_code);
    info!("Added lobby {lobby_code}");

    Json(CreateLobbyResponse {
        lobby_code: lobby_code,
    })
}

pub async fn handle_geo_guessr(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (lobby_code, player_username) =
        match GeoGuessr::await_join_req(&mut receiver, &mut sender).await {
            Ok((lobby_code, player_username)) => (lobby_code, player_username),
            Err(_) => return,
        };

    let game_obj = match state.games.geo_guessr.get(&lobby_code) {
        Some(game) => game.clone(),
        None => {
            info!("Lobby not found: {}", lobby_code);
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&GeoGuessrServerEvent::LobbyEvent(
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

    let player_id = game_obj.lobby.lock().unwrap().get_new_player_id();

    let connection_span = tracing::info_span!(
        "connection",
        lobby=%lobby_code,
        player=%player_id,
    );

    match game_obj.player_join(player_id.clone(), player_username.clone()) {
        Ok(_) => {
            info!("Player: {}, joined lobby: {}", player_id, lobby_code);
        }
        Err(e) => {
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&GeoGuessrServerEvent::LobbyEvent(
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

    // Clone the broadcast channel into tx (Sender)
    let tx = game_obj.broadcast.clone();
    // Subscribe to the broadcast channel to aquire a (Receiver)
    let mut rx = tx.subscribe();

    // Extract the gamestate
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&GeoGuessrServerEvent::GameEvent(
                GeoGuessrGameEvent::SyncState {
                    players: game_obj.get_players(),
                    settings: game_obj.get_settings(),
                    leaderboard: game_obj.get_leaderboard(),
                    status: game_obj.get_lobby_status(),
                },
            ))
            .expect("Failed to parse SyncState event")
            .into(),
        ))
        .await;

    // Create the send_task
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

    let _ = game_obj.broadcast.send(GeoGuessrServerEvent::LobbyEvent(
        LobbyServerEvent::PlayerJoin {
            player_id: player_id.clone(),
            player_username: player_username.clone(),
        },
    ));

    // Create the receive task
    let mut recv_task = tokio::spawn(
        async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(req) => {
                        let event = match serde_json::from_str::<GeoGuesserClientEvent>(&req) {
                            Ok(e) => e,
                            Err(e) => {
                                warn!("Failed to parse client event: {:?}", e);
                                continue;
                            }
                        };
                        match event {
                            GeoGuesserClientEvent::LobbyEvent(e) => {
                                game_obj.handle_lobby_event(player_id, e);
                            }
                            GeoGuesserClientEvent::GameEvent(e) => {
                                game_obj.handle_game_event(player_id, e);
                            }
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        .instrument(connection_span),
    );

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    info!("Websocket disconnected");
}
