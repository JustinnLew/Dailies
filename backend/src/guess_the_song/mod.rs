use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    AppState, generate_lobby_code,
    state::{GuessTheSongGame, GuessTheSongServerEvent, GuessTheSongUserEvent, LobbyStatus},
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
use tokio::{
    sync::{broadcast, mpsc},
    time::{Duration, Instant, sleep},
};
use tracing::{Instrument, info, instrument, warn};
use uuid::Uuid;
pub mod api;

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}
struct GuessTheSongConnectionGuard {
    game: Arc<GuessTheSongGame>,
    player_id: Uuid,
    broadcast: broadcast::Sender<GuessTheSongServerEvent>,
    cleanup_tx: mpsc::UnboundedSender<String>,
}

impl Drop for GuessTheSongConnectionGuard {
    fn drop(&mut self) {
        let mut lobby_state = self.game.lobby_state.lock().unwrap();
        lobby_state.player_leave(&self.player_id);
        info!(
            "Player {} disconnected from lobby: {}",
            self.player_id, self.game.lobby_code
        );
        let _ = self.broadcast.send(GuessTheSongServerEvent::PlayerLeave {
            player_id: self.player_id.clone(),
        });
        if lobby_state.players.is_empty() {
            info!(
                "Lobby {} is now empty, scheduling for cleanup",
                self.game.lobby_code
            );
            let _ = self.cleanup_tx.send(self.game.lobby_code.clone());
        }
    }
}

#[instrument(name = "CREATE LOBBY", skip(state))]
pub async fn guess_the_song_create_lobby(State(state): State<AppState>) -> impl IntoResponse {
    let mut lobby_code;
    loop {
        lobby_code = generate_lobby_code();
        if !state.games.valid_lobby_code(&lobby_code) {
            break;
        }
    }
    state.games.add_guess_the_song_lobby(&lobby_code);
    info!("Added lobby {lobby_code}");

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
    // 1. Deserialize to the Enum Type
    let event = match serde_json::from_str::<GuessTheSongUserEvent>(&join_req) {
        Ok(e) => e,
        Err(_) => {
            info!("JOIN ERROR");
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&GuessTheSongServerEvent::JoinError {
                        message: "Failed to serialize Inital Request".to_string(),
                    })
                    .unwrap()
                    .into(),
                ))
                .await;
            return;
        }
    };

    // 2. Use Pattern Matching to extract the fields from the Join variant
    let (lobby_code, player_username) = match event {
        GuessTheSongUserEvent::Join {
            lobby_code,
            username,
        } => (lobby_code, username),
        _ => {
            info!("JOIN ERROR");
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&GuessTheSongServerEvent::JoinError {
                        message: "Expected Join Event".to_string(),
                    })
                    .unwrap()
                    .into(),
                ))
                .await;
            return;
        }
    };

    let game_obj = match state.games.guess_the_song.get(&lobby_code) {
        Some(g) => g.clone(),
        None => {
            info!("JOIN ERROR");
            let _ = sender
                .send(Message::Text(
                    serde_json::to_string(&GuessTheSongServerEvent::JoinError {
                        message: "Lobby not found".to_string(),
                    })
                    .unwrap()
                    .into(),
                ))
                .await;
            return;
        }
    };

    let player_id = game_obj.get_new_player_id();

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
                    serde_json::to_string(&GuessTheSongServerEvent::JoinError {
                        message: e.to_string(),
                    })
                    .unwrap()
                    .into(),
                ))
                .await;
            return;
        }
    }
    let _guard = GuessTheSongConnectionGuard {
        game: game_obj.clone(),
        player_id: player_id.clone(),
        broadcast: game_obj.broadcast.clone(),
        cleanup_tx: state.cleanup.clone(),
    };
    // Clone the broadcast channel into tx (Sender)
    let tx = game_obj.broadcast.clone();
    // Subscribe to the broadcast channel to aquire a (Receiver)
    let mut rx = tx.subscribe();

    // Extract the gamestate
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&GuessTheSongServerEvent::SyncState {
                players: game_obj.get_players(),
                settings: game_obj.get_settings(),
                leaderboard: game_obj.get_leaderboard(),
                preview_url: game_obj.get_current_song().map(|s| s.url),
                status: game_obj.get_lobby_status(),
                round_start_time: game_obj.get_round_start_time(),
            })
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

    let _ = game_obj
        .broadcast
        .send(GuessTheSongServerEvent::PlayerJoin {
            player_id: player_id.clone(),
            player_username: player_username.clone(),
        });

    //  Create the recv_task
    let mut prev_guess_time_stamp = Instant::now();
    let mut recv_task = tokio::spawn(
        async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(req) => {
                        println!("{:?}", req);
                        let req: GuessTheSongUserEvent = match serde_json::from_str(&req) {
                            Ok(r) => r,
                            Err(e) => {
                                warn!("Failed to parse user event: {:?}", e);
                                continue;
                            }
                        };
                        match req {
                            GuessTheSongUserEvent::Ready => {
                                game_obj.player_ready(&player_id);
                                info!("READY");
                                let _ =
                                    game_obj
                                        .broadcast
                                        .send(GuessTheSongServerEvent::PlayerReady {
                                            player_id: player_id.clone(),
                                        });
                                if game_obj.all_ready()
                                    && game_obj.lobby_state.lock().unwrap().status
                                        == LobbyStatus::Waiting
                                {
                                    let _ =
                                        game_obj.broadcast.send(GuessTheSongServerEvent::AllReady);
                                    let playlist_link = game_obj.get_playlist_link();
                                    let l = game_obj.clone();
                                    let spotify_client = state.spotify_client.clone();
                                    let player_id_c = player_id.clone();
                                    tokio::spawn(async move {
                                        let res = api::load_songs(
                                            &spotify_client,
                                            &playlist_link,
                                            l.clone(),
                                        )
                                        .await;
                                        match res {
                                            Ok(_) => {
                                                l.update_lobby_status(LobbyStatus::Playing);
                                            }
                                            Err(msg) => {
                                                l.update_lobby_status(LobbyStatus::Waiting);
                                                l.player_unready(&player_id_c);
                                                let _ = l.broadcast.send(
                                                    GuessTheSongServerEvent::PlayerUnready {
                                                        player_id: player_id_c,
                                                    },
                                                );
                                                let _ = l.broadcast.send(
                                                    GuessTheSongServerEvent::PlaylistError {
                                                        message: msg,
                                                    },
                                                );
                                                return;
                                            }
                                        }
                                        run_guess_the_song_game(l).await;
                                    });
                                }
                            }
                            GuessTheSongUserEvent::Unready => {
                                info!("UNREADY");
                                game_obj.player_unready(&player_id);
                                let _ = game_obj.broadcast.send(
                                    GuessTheSongServerEvent::PlayerUnready {
                                        player_id: player_id.clone(),
                                    },
                                );
                            }
                            GuessTheSongUserEvent::UpdateGameSettings { settings } => {
                                info!("UPDATE SETTINGS: {:?}", settings);
                                game_obj.update_game_settings(settings.clone());
                                let _ = game_obj.broadcast.send(
                                    GuessTheSongServerEvent::GameSettingsUpdated {
                                        settings: settings,
                                    },
                                );
                            }
                            GuessTheSongUserEvent::Guess { content } => {
                                info!(guess=%content, "GUESS:");
                                let cur_guess_time_stamp = Instant::now();
                                if cur_guess_time_stamp.duration_since(prev_guess_time_stamp).as_secs() < game_obj.get_settings().answer_delay_seconds {
                                    let _ =
                                        game_obj
                                        .broadcast
                                        .send(GuessTheSongServerEvent::PlayerGuess {
                                            username: "ERROR".to_string(),
                                            content: content.clone(),
                                        });
                                    continue;
                                } else {
                                    let _ =
                                        game_obj
                                        .broadcast
                                        .send(GuessTheSongServerEvent::PlayerGuess {
                                            username: player_username.clone(),
                                            content: content.clone(),
                                        });
                                }
                                prev_guess_time_stamp = cur_guess_time_stamp;
                                if let Some(s) = game_obj.is_correct_song(&content) {
                                    game_obj.increment_player_score(&player_id, 2);
                                    info!(guess=%content, "CORRECT SONG:");
                                    let _ = game_obj.broadcast.send(
                                        GuessTheSongServerEvent::CorrectGuess {
                                            player_id: player_id.clone(),
                                            msg: format!(
                                                "{} guessed the song correctly! The song was '{}'.",
                                                player_username, s
                                            ),
                                        },
                                    );
                                }
                                if let Some(a) = game_obj.is_correct_artist(&content) {
                                    game_obj.increment_player_score(&player_id, 2);
                                    info!(guess=%content, "CORRECT ARTIST:");
                                    let _ = game_obj.broadcast.send(
                                    GuessTheSongServerEvent::CorrectGuess {
                                        player_id: player_id.clone(),
                                        msg: format!(
                                            "{} guessed the artist correctly! The artist was '{}'.",
                                            player_username, a
                                        ),
                                    },
                                );
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
        }
        .instrument(connection_span),
    );

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    info!("Websocket disconnected");
}

#[instrument(name="GAME LOOP", skip(game), fields(lobby=%game.lobby_code))]
async fn run_guess_the_song_game(game: Arc<GuessTheSongGame>) {
    info!("Starting Guess The Song game");
    let _ = game.broadcast.send(GuessTheSongServerEvent::GameStart);
    sleep(Duration::from_secs(3)).await;
    let settings = {
        let s = game.settings.lock().unwrap();
        s.clone()
    };

    for _ in 0..settings.num_songs {
        if game.lobby_state.lock().unwrap().empty() {
            info!("Game empty, terminating loop");
            return;
        }
        let (song, round_start_time) = {
            let mut state = game.state.lock().unwrap();
            match state.get_next_song() {
                Some(s) => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    state.round_start_time = Some(now);
                    (s, now)
                }
                None => {
                    info!("No songs left, ending game");
                    let _ = game.broadcast.send(GuessTheSongServerEvent::GameEnd);
                    break;
                }
            }
        };

        info!(song=%song.title, "ROUNDSTART:");
        let _ = game.broadcast.send(GuessTheSongServerEvent::RoundStart {
            preview_url: song.url.clone(),
            round_start_time,
        });

        sleep(Duration::from_secs(settings.round_length_seconds as u64)).await;

        info!("ROUNDEND");
        let _ = game.broadcast.send(GuessTheSongServerEvent::RoundEnd {
            correct_title: song.title.clone(),
            correct_artists: song.artists.clone(),
            leaderboard: game.get_leaderboard(),
        });
        sleep(Duration::from_secs(settings.round_delay_seconds as u64)).await;
    }
    info!("GAME END");
    let _ = game.broadcast.send(GuessTheSongServerEvent::GameEnd);
    game.reset();
}
