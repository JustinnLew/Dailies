use std::sync::Arc;

use crate::{
    AppState, generate_lobby_code, spotify::load_songs, state::{GameSettings, GameState, Lobby, LobbyStatus, ServerEvent}
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
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

#[derive(serde::Serialize)]
struct CreateLobbyResponse {
    lobby_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GuessTheSongGameSettings {
    playlist_link: String,
    num_songs: u8,
}

impl GuessTheSongGameSettings {
    pub fn new() -> Self {
        Self {
            playlist_link: String::new(),
            num_songs: 10,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "event")]
enum GuessTheSongUserEvent {
    Join {
        lobby_code: String,
        user_id: String,
        username: String,
    },
    Ready,
    UpdateGameSettings {
        settings: GuessTheSongGameSettings,
    },
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
    let lobby = match state.games.get_lobby(&lobby_code) {
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
                num_songs: lobby.get_game_settings().num_songs,
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
                    println!("Received message from {}: {}", player_id, req);
                    let req: GuessTheSongUserEvent = match serde_json::from_str(&req) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed to parse user event: {:?}", e);
                            continue;
                        }
                    };
                    println!("{:?}", req);
                    match req {
                        GuessTheSongUserEvent::Ready => {
                            lobby.player_ready(&player_id);
                            let _ = lobby.broadcast.send(ServerEvent::PlayerReady {
                                player_id: player_id.clone(),
                            });
                            if lobby.all_ready() {
                                let _ = lobby.broadcast.send(ServerEvent::AllReady);
                                lobby.update_lobby_status(LobbyStatus::Playing);
                                let playlist_link = match &lobby.get_game_settings() {
                                    GameSettings::GuessTheSong(settings) => settings.playlist_link.clone(),
                                    // _ => return,
                                };

                                let l = lobby.clone();
                                let spotify_client = state.spotify_client.clone();
                                tokio::spawn(async move {
                                    load_songs(&spotify_client, &playlist_link, l.clone()).await;
                                    run_guess_the_song_game(l).await;
                                });
                            }
                        }
                        GuessTheSongUserEvent::UpdateGameSettings { settings } => {
                            lobby.update_game_settings(&GameSettings::GuessTheSong(
                                settings.clone(),
                            ));
                            let _ = lobby.broadcast.send(ServerEvent::GameSettingsUpdated {
                                settings: GameSettings::GuessTheSong(settings),
                            });
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

async fn run_guess_the_song_game(lobby: Arc<Lobby>) {
    println!("Starting Guess The Song game");

    loop {
        let song = {
            let mut state = lobby.state.lock().unwrap();

            let songs = match &mut state.game {
                GameState::GuessTheSong { songs, .. } => songs,
                _ => { return; }
            };

            if songs.is_empty() {
                println!("No songs left, ending game");
                // state.status = LobbyStatus::Finished;
                let _ = lobby.broadcast.send(ServerEvent::GameEnd);
                break;
            }
            songs.pop().unwrap()
        };

        // --- 2. Broadcast song start ---
        let _ = lobby.broadcast.send(ServerEvent::RoundStart {
            preview_url: song.url.clone(),
        });

        // --- 3. Wait for round duration ---
        sleep(Duration::from_secs(30)).await;

        // --- 4. End round ---
        let _ = lobby.broadcast.send(ServerEvent::RoundEnd {
            correct_title: song.title.clone(),
            correct_artists: song.artists.clone(),
        });
    }
}
