use std::{collections::HashMap, sync::{Arc, Mutex}};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{broadcast},
    time::{Duration, sleep},
};
use tracing::info;
use uuid::Uuid;

use crate::{connections::ConnectionManager, state::{GuessTheSongServerEvent, LobbyServerEvent, LobbyState, LobbyStatus, LobbyUserEvent}};

/// ===============================================
/// Main Parent Struct for Guess The Song Game
/// ===============================================
pub(crate) struct GeoGuessr {
    pub lobby: Mutex<LobbyState>,
    pub broadcast: broadcast::Sender<GeoGuessrServerEvent>,
    pub settings: Mutex<GeoGuessrSettings>,
    pub state: Mutex<GeoGuessrState>,
    pub lobby_code: String,
}

impl GeoGuessr {
    fn reset(&self) {
        self.state.lock().unwrap().reset();
    }

    pub async fn await_join_req(receiver: &mut SplitStream<WebSocket>, sender: &mut SplitSink<WebSocket, Message>) -> Result<(String, String), ()>{
        let join_req = match receiver.next().await {
            Some(Ok(Message::Text(m))) => m,
            _ => return Err(()),
        };
        let event = match serde_json::from_str::<GeoGuesserClientEvent>(&join_req) {
            Ok(e) => e,
            Err(_) => {
                info!("JOIN ERROR");
                let _ = sender
                    .send(Message::Text(
                        serde_json::to_string(&GeoGuessrServerEvent::LobbyEvent(LobbyServerEvent::JoinError {
                            message: "Failed to serialize Inital Request".to_string(),
                        }))
                        .unwrap()
                        .into(),
                    ))
                    .await;
                return Err(());
            }
        };

        let (lobby_code, player_username) = match event {
            GeoGuesserClientEvent::LobbyEvent(LobbyUserEvent::Join { lobby_code, username }) => (lobby_code, username),
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
                return Err(());
            }
        };
        Ok((lobby_code, player_username))
    }

    pub fn player_join(
        &self,
        player_id: Uuid,
        player_username: String,
    ) -> Result<(), &str> {
        let mut lobby = self.lobby.lock().unwrap();
        let mut state = self.state.lock().unwrap();
        if lobby.status != crate::state::LobbyStatus::Waiting {
            return Err("Cannot join game in progress");
        }
        lobby.player_join(player_id.clone(), player_username);
        state.scores.insert(player_id.clone(), 0);
        Ok(())
    }

    pub fn get_players(&self) -> Vec<(Uuid, String, bool)> {
        self.lobby.lock().unwrap().get_players()
    }

    pub fn get_settings(&self) -> GeoGuessrSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn get_leaderboard(&self) -> HashMap<Uuid, u32> {
        let state = self.state.lock().unwrap();
        state.scores.clone()
    }

    pub fn get_lobby_status(&self) -> LobbyStatus {
        self.lobby.lock().unwrap().status.clone()
    }

    pub fn update_lobby_status(&self, status: LobbyStatus) {
        let mut lobby = self.lobby.lock().unwrap();
        lobby.status = status;
    }

    async fn load_locations(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn handle_game_event(&self, player_id: Uuid, event: GeoGuessrUserGameEvent) {
        match event {
            GeoGuessrUserGameEvent::UpdateGameSettings { settings } => {
                let lobby = self.lobby.lock().unwrap();
                if lobby.status != crate::state::LobbyStatus::Waiting {
                    return;
                }
                self.settings.lock().unwrap().update_game_settings(settings.clone());
                let _ = self.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::GameSettingsUpdated {
                    settings
                }));
            },
            GeoGuessrUserGameEvent::Guess { lat, lng } => {
                let _ = self.broadcast.send(
                    GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::PlayerGuess {
                        player_id: player_id.clone(),
                        lat,
                        lng,
                    },
                ));
            },
            _ => { return; }
       }
    }

    pub fn handle_lobby_event(self: &Arc<Self>, player_id: Uuid, event: LobbyUserEvent) {
        match event {
            LobbyUserEvent::Ready => {
                self.lobby.lock().unwrap().player_ready(&player_id);
                let _ = self.broadcast.send(
                    GeoGuessrServerEvent::LobbyEvent(LobbyServerEvent::PlayerReady {
                        player_id: player_id.clone(),
                    },
                ));
                if self.lobby.lock().unwrap().all_ready() && self.lobby.lock().unwrap().status == LobbyStatus::Waiting {
                    let _ = self.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::AllReady));
                    let l = Arc::clone(self);
                    tokio::spawn(async move {
                        let res = l.load_locations().await;
                        match res {
                            Ok(_) => {l.update_lobby_status(LobbyStatus::Playing); }
                            Err(e) => {
                                l.update_lobby_status(LobbyStatus::Waiting);
                                l.lobby.lock().unwrap().player_unready(&player_id);
                                let _ = l.broadcast.send(GeoGuessrServerEvent::LobbyEvent(LobbyServerEvent::PlayerUnready {
                                    player_id: player_id,
                                }));
                                let _ = l.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::LoadingError {
                                    message: format!("Failed to load locations: {}", e),
                                }));
                                return;
                            }
                        }
                        GeoGuessr::run_game(l).await;
                    });
                }
            },
            LobbyUserEvent::Unready => {
                self.lobby.lock().unwrap().player_unready(&player_id);
                let _ = self.broadcast.send(
                    GeoGuessrServerEvent::LobbyEvent(LobbyServerEvent::PlayerUnready {
                        player_id: player_id.clone(),
                    },
                ));
            },
            _ => { return; }
        }
    }

    pub async fn run_game(game: Arc<GeoGuessr>) {
        info!("Starting GeoGuessr game");
        let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::GameStart));
        let settings = {
            let s = game.settings.lock().unwrap();
            s.clone()
        };
        sleep(Duration::from_secs(3)).await;

        for _ in 0..settings.num_rounds {
            if game.lobby.lock().unwrap().empty() {
                info!("Game empty, terminating loop");
                return;
            }
            let location= {
                let mut state = game.state.lock().unwrap();
                match state.get_next_location() {
                    Some(s) => {s}
                    None => {
                        info!("No locations left, ending game");
                        let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::GameEnd));
                        break;
                    }
                }
            };

            info!(location=%location.image_id, "ROUNDSTART:");
            let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::RoundStart {
                image_id: location.image_id.clone(),
            }));
            sleep(Duration::from_secs(settings.round_length_seconds as u64)).await;
            info!("ROUNDEND");

             let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::RoundEnd {
                correct_lat: location.lat,
                correct_lng: location.lng,
                leaderboard: game.get_leaderboard(),
            }));
            sleep(Duration::from_secs(settings.round_delay_seconds as u64)).await;
        }

        info!("GAME END");
        if (settings.round_delay_seconds as u64) < 3 {
            sleep(Duration::from_secs(3 - settings.round_delay_seconds as u64)).await;
        }
        let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::GameEnd));
        game.reset();
    }
}

impl ConnectionManager for GeoGuessr {
    fn connection_drop(&self, player_id: Uuid) {
        self.lobby.lock().unwrap().player_leave(&player_id);
        info!(
            "Player {} disconnected from lobby: {}",
            player_id, self.lobby_code
        );
        let _ = self.broadcast.send(GeoGuessrServerEvent::LobbyEvent(LobbyServerEvent::PlayerLeave {
            player_id: player_id.clone(),
        }));
    }
    fn lobby_code(&self) -> String {
        return self.lobby_code.clone()
    }
    fn no_connections(&self) -> bool {
        self.lobby.lock().unwrap().empty()
    }
}

/// ===============================================
/// Settings
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeoGuessrSettings {
    pub num_rounds: u8,
    pub round_length_seconds: u8,
    pub round_delay_seconds: u8,
}

impl GeoGuessrSettings {
    pub fn new() -> Self {
        Self {
            num_rounds: 10,
            round_length_seconds: 30,
            round_delay_seconds: 3,
        }
    }

    pub fn get_num_rounds(&self) -> u8 {
        self.num_rounds
    }

    pub fn update_game_settings(&mut self, settings: GeoGuessrSettings) {
        self.num_rounds = settings.num_rounds;
        self.round_length_seconds = settings.round_length_seconds;
        self.round_delay_seconds = settings.round_delay_seconds;
    }
}

/// ===============================================
/// State
/// ===============================================
pub(crate) struct GeoGuessrState {
    pub scores: HashMap<Uuid, u32>,
    pub locations: Vec<Location>,
    pub location_index: usize,
}

impl GeoGuessrState {
    pub fn new() -> Self {
        GeoGuessrState {
            scores: HashMap::new(),
            locations: Vec::new(),
            location_index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.scores.iter_mut().for_each(|(_, score)| *score = 0);
        self.location_index = 0;
        self.locations = Vec::new();
    }

    pub fn get_current_location(&self) -> Option<Location> {
        if self.locations.is_empty() || self.location_index == 0 || self.location_index - 1 >= self.locations.len()
        {
            return None;
        }
        Some(Location {
            image_id: self.locations[self.location_index - 1].image_id.clone(),
            lat: self.locations[self.location_index - 1].lat,
            lng: self.locations[self.location_index - 1].lng,
        })
    }

    pub fn add_location(&mut self, location: Location) {
        self.locations.push(location);
    }

    pub fn get_next_location(&mut self) -> Option<Location> {
        self.location_index += 1;
        self.get_current_location()
    }

    pub fn increment_player_score(&mut self, player_id: &Uuid, points: u32) {
        if let Some(score) = self.scores.get_mut(player_id) {
            *score += points;
        }
    }
}

/// ===============================================
/// Server Events
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event")]
pub(crate) enum GeoGuessrGameEvent {
    SyncState {
        players: Vec<(Uuid, String, bool)>,
        settings: GeoGuessrSettings,
        leaderboard: HashMap<Uuid, u32>,
        status: LobbyStatus,
    },
    AllReady,
    GameStart,
    GameSettingsUpdated {
        settings: GeoGuessrSettings,
    },
    RoundStart {
        image_id: String,
    },
    RoundEnd {
        correct_lat: f32,
        correct_lng: f32,
        leaderboard: HashMap<Uuid, u32>,
    },
    GameEnd,
    PlayerGuess {
        player_id: Uuid,
        lat: f32,
        lng: f32,
    },
    LoadingError {
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub(crate) enum GeoGuessrServerEvent {
    LobbyEvent(LobbyServerEvent),
    GameEvent(GeoGuessrGameEvent),
}

/// ===============================================
/// User Events
/// ===============================================
#[derive(Deserialize, Debug)]
#[serde(tag = "event")]
pub(crate) enum GeoGuessrUserGameEvent {
    UpdateGameSettings {
        settings: GeoGuessrSettings,
    },
    Guess {
        lat: f32,
        lng: f32,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub(crate) enum GeoGuesserClientEvent {
    LobbyEvent(LobbyUserEvent),
    GameEvent(GeoGuessrUserGameEvent),
}
/// ===============================================
/// Helper Structs
/// ===============================================
#[derive(Debug, Clone)]
pub(crate) struct Location {
    pub image_id: String,
    pub lat: f32,
    pub lng: f32,
}
