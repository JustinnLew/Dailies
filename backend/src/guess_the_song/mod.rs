use std::sync::{Arc};

use crate::{
    AppState, generate_lobby_code,
    state::{
        GuessTheSongGame, GuessTheSongGameSettings, GuessTheSongServerEvent, GuessTheSongUserEvent,
        LobbyStatus,
    },
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
    time::{Duration, sleep},
};
pub mod api;

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}
struct GuessTheSongConnectionGuard<'a> {
    game: Arc<GuessTheSongGame>,
    player_id: String,
    broadcast: broadcast::Sender<GuessTheSongServerEvent>,
    cleanup_tx: mpsc::UnboundedSender<String>,
    lobby_code: &'a String,
}

impl<'a> Drop for GuessTheSongConnectionGuard<'a> {
    fn drop(&mut self) {
        let mut lobby_state = self.game.lobby_state.lock().unwrap();
        lobby_state.player_leave(&self.player_id);
        println!(
            "Player {} disconnected and removed from lobby",
            self.player_id
        );
        let _ = self.broadcast.send(GuessTheSongServerEvent::PlayerLeave {
            player_id: self.player_id.clone(),
        });
        if lobby_state.players.is_empty() {
            println!(
                "Lobby {} is now empty, scheduling for cleanup",
                self.lobby_code
            );
            let _ = self.cleanup_tx.send(self.lobby_code.clone());
        }
    }
}

pub async fn guess_the_song_create_lobby(State(state): State<AppState>) -> impl IntoResponse {
    let mut lobby_code;
    loop {
        lobby_code = generate_lobby_code();
        if !state.games.valid_lobby_code(&lobby_code) {
            break;
        }
    }
    state.games.add_guess_the_song_lobby(&lobby_code);
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
    // 1. Deserialize to the Enum Type
    let event = match serde_json::from_str::<GuessTheSongUserEvent>(&join_req) {
        Ok(e) => e,
        Err(_) => {
            let _ = sender.send(Message::Text("Invalid JSON".into())).await;
            return;
        }
    };

    // 2. Use Pattern Matching to extract the fields from the Join variant
    let (lobby_code, player_id, player_username) = match event {
        GuessTheSongUserEvent::Join {
            lobby_code,
            user_id,
            username,
        } => (lobby_code, user_id, username),
        _ => {
            let _ = sender
                .send(Message::Text("Expected join event".into()))
                .await;
            return;
        }
    };

    let game_obj = match state.games.guess_the_song.get(&lobby_code) {
        Some(g) => g.clone(),
        None => {
            let _ = sender.send(Message::Text("Lobby Not Found".into())).await;
            return;
        }
    };

    match game_obj.player_join(player_id.clone(), player_username.clone()) {
        Ok(_) => {}
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
        lobby_code: &lobby_code,
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
                num_songs: game_obj.get_num_songs(),
                playlist_link: game_obj.get_playlist_link(),
                round_length_seconds: game_obj.get_round_length_seconds(),
                answer_delay_seconds: game_obj.get_answer_delay_seconds(),
                round_delay_seconds: game_obj.get_round_delay_seconds(),
            })
            .expect("Failed to parse SyncState event")
            .into(),
        ))
        .await;

    // Create the send_task
    let mut send_task: tokio::task::JoinHandle<()> = tokio::spawn(async move {
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

    let _ = game_obj
        .broadcast
        .send(GuessTheSongServerEvent::PlayerJoin {
            player_id: player_id.clone(),
            player_username: player_username.clone(),
        });

    //  Create the recv_task
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(req) => {
                    println!("Received message from {}: {}", player_id, req);
                    let req: GuessTheSongUserEvent = match serde_json::from_str(&req) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed to parse user event: {:?}", e);
                            continue;
                        }
                    };
                    match req {
                        GuessTheSongUserEvent::Ready => {
                            game_obj.player_ready(&player_id);
                            let _ = game_obj
                                .broadcast
                                .send(GuessTheSongServerEvent::PlayerReady {
                                    player_id: player_id.clone(),
                                });
                            if game_obj.all_ready() {
                                let _ = game_obj.broadcast.send(GuessTheSongServerEvent::AllReady);
                                game_obj.update_lobby_status(LobbyStatus::Playing);
                                let playlist_link = game_obj.get_playlist_link();

                                let l = game_obj.clone();
                                let spotify_client = state.spotify_client.clone();
                                tokio::spawn(async move {
                                    api::load_songs(&spotify_client, &playlist_link, l.clone())
                                        .await;
                                    run_guess_the_song_game(l).await;
                                });
                            }
                        }
                        GuessTheSongUserEvent::UpdateGameSettings { settings } => {
                            game_obj.update_game_settings(GuessTheSongGameSettings {
                                playlist_link: settings.playlist_link.clone(),
                                num_songs: settings.num_songs,
                                round_length_seconds: settings.round_length_seconds,
                                answer_delay_seconds: settings.answer_delay_seconds,
                                round_delay_seconds: settings.round_delay_seconds,
                            });
                            let _ = game_obj.broadcast.send(
                                GuessTheSongServerEvent::GameSettingsUpdated {
                                    settings: GuessTheSongGameSettings {
                                        playlist_link: settings.playlist_link.clone(),
                                        num_songs: settings.num_songs,
                                        round_length_seconds: settings.round_length_seconds,
                                        answer_delay_seconds: settings.answer_delay_seconds,
                                        round_delay_seconds: settings.round_delay_seconds,
                                    },
                                },
                            );
                        }
                        GuessTheSongUserEvent::Guess { content } => {
                            let _ = game_obj
                                .broadcast
                                .send(GuessTheSongServerEvent::PlayerGuess {
                                    username: player_username.clone(),
                                    content: content.clone(),
                                });
                            if let Some(s) = game_obj.is_correct_song(&content) {
                                game_obj.increment_player_score(&player_id, 2);
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
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    println!("Websocket disconnected");
}

async fn run_guess_the_song_game(game: Arc<GuessTheSongGame>) {
    println!("Starting Guess The Song game");
    let _ = game.broadcast
        .send(GuessTheSongServerEvent::GameStart);
    sleep(Duration::from_secs(3)).await;
    let settings = {
        let s = game.settings.lock().unwrap();
        s.clone()
    };

    for _ in 0..settings.num_songs {
        let song = {
            let mut state = game.state.lock().unwrap();
            match state.get_next_song() {
                Some(s) => s,
                None => {
                    println!("No songs left, ending game");
                    let _ = game.broadcast.send(GuessTheSongServerEvent::GameEnd);
                    break;
                }
            }
        };

        // --- 2. Broadcast song start ---
        let _ = game.broadcast.send(GuessTheSongServerEvent::RoundStart {
            preview_url: song.url.clone(),
        });

        // --- 3. Wait for round duration ---
        sleep(Duration::from_secs(settings.round_length_seconds as u64)).await;

        // --- 4. End round ---
        let _ = game.broadcast.send(GuessTheSongServerEvent::RoundEnd {
            correct_title: song.title.clone(),
            correct_artists: song.artists.clone(),
            leaderboard: game.get_leaderboard(),
        });
        sleep(Duration::from_secs(settings.round_delay_seconds as u64)).await;
    }
    let _ = game.broadcast.send(GuessTheSongServerEvent::GameEnd);
}
