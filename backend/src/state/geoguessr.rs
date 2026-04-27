use std::{collections::HashMap, sync::Mutex};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
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

    pub fn handle_game_event(&self, player_id: Uuid, player_username: String, event: GeoGuessrUserGameEvent) {
        match event {
            GeoGuessrUserGameEvent::UpdateGameSettings { settings } => {
                let mut lobby = self.lobby.lock().unwrap();
                if lobby.status != crate::state::LobbyStatus::Waiting {
                    return;
                }
                self.settings.lock().unwrap().update_game_settings(settings.clone());
                let _ = self.broadcast.send(GeoGuessrServerEvent::GeoGuessrGameEvent(GeoGuessrGameEvent::GameSettingsUpdated {
                    settings
                }));
            },
            GeoGuessrUserGameEvent::Guess { lat, lng } => {

            },
            _ => { return; }
       }
    }

    pub fn handle_lobby_event(&self, player_id: Uuid, player_username: String, event: LobbyUserEvent) {
        match event {
            LobbyUserEvent::Ready => {
                // TODO
            },
            LobbyUserEvent::Unready => {
                // TODO
            },
            _ => { return; }
        }
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
#[serde(tag = "event", content = "data")]
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
        username: String,
        lat: f32,
        lng: f32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event", content = "data")]
pub(crate) enum GeoGuessrServerEvent {
    LobbyEvent(LobbyServerEvent),
    GeoGuessrGameEvent(GeoGuessrGameEvent),
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
#[serde(tag = "event", content = "data")]
pub(crate) enum GeoGuesserClientEvent {
    LobbyEvent(LobbyUserEvent),
    GeoGuessrUserGameEvent(GeoGuessrUserGameEvent),
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
