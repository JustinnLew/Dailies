use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{Notify, broadcast},
    time::{Duration, sleep},
};
use tracing::info;
use uuid::Uuid;

use crate::{
    connections::ConnectionManager,
    state::{GuessTheSongServerEvent, LobbyServerEvent, LobbyState, LobbyStatus, LobbyUserEvent},
};

struct Map {
    center: (f32, f32),
    file: String,
}

static MAPS: LazyLock<HashMap<String, Map>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "World".to_string(),
        Map {
            center: (0.0, 0.0),
            file: "world.json".to_string(),
        },
    );
    m.insert(
        "Australian Cities".to_string(),
        Map {
            center: (-25.2744, 133.7751),
            file: "australian_cities.json".to_string(),
        },
    );
    m.insert(
        "Sydney".to_string(),
        Map {
            center: (-25.2744, 133.7751),
            file: "sydney.json".to_string(),
        },
    );
    m
});

/// ===============================================
/// Main Parent Struct for GeoGuessr Game
/// ===============================================
pub(crate) struct GeoGuessr {
    pub lobby: Mutex<LobbyState>,
    pub broadcast: broadcast::Sender<GeoGuessrServerEvent>,
    pub settings: Mutex<GeoGuessrSettings>,
    pub state: Mutex<GeoGuessrState>,
    pub lobby_code: String,
    pub round_notify: Mutex<Arc<Notify>>,
}

impl GeoGuessr {
    fn reset(&self) {
        self.state.lock().unwrap().reset();
        self.lobby.lock().unwrap().reset();
    }

    pub async fn await_join_req(
        receiver: &mut SplitStream<WebSocket>,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> Result<(String, String), ()> {
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
                        serde_json::to_string(&GeoGuessrServerEvent::LobbyEvent(
                            LobbyServerEvent::JoinError {
                                message: "Failed to serialize Inital Request".to_string(),
                            },
                        ))
                        .unwrap()
                        .into(),
                    ))
                    .await;
                return Err(());
            }
        };

        let (lobby_code, player_username) = match event {
            GeoGuesserClientEvent::LobbyEvent(LobbyUserEvent::Join {
                lobby_code,
                username,
            }) => (lobby_code, username),
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

    pub fn player_join(&self, player_id: Uuid, player_username: String) -> Result<(), &str> {
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
        let mut rng = rand::rng();
        let settings = self.settings.lock().unwrap().clone();

        let map = MAPS
            .get(&settings.map)
            .ok_or_else(|| format!("Unknown map: {}", settings.map))?;

        let all_locations: Vec<Location> = {
            let path = format!("../geo_data/{}.json", map.file);
            let data = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", path, e))?;
            serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse {}: {}", path, e))?
        };

        let sample: Vec<Location> = all_locations
            .choose_multiple(&mut rng, settings.num_rounds as usize)
            .cloned()
            .collect();

        let mut state = self.state.lock().unwrap();
        state.locations = sample;
        Ok(())
    }

    pub fn handle_game_event(&self, player_id: Uuid, event: GeoGuessrUserGameEvent) {
        match event {
            GeoGuessrUserGameEvent::UpdateGameSettings { settings } => {
                let lobby = self.lobby.lock().unwrap();
                if lobby.status != crate::state::LobbyStatus::Waiting {
                    return;
                }
                self.settings
                    .lock()
                    .unwrap()
                    .update_game_settings(settings.clone());
                let _ = self.broadcast.send(GeoGuessrServerEvent::GameEvent(
                    GeoGuessrGameEvent::GameSettingsUpdated { settings: self.settings.lock().unwrap().clone() },
                ));
            }
            GeoGuessrUserGameEvent::Guess { lat, lng } => {
                if self.lobby.lock().unwrap().status != LobbyStatus::Playing {
                    return;
                }

                let player_count = self.lobby.lock().unwrap().player_count();
                let all_guessed = {
                    let mut state = self.state.lock().unwrap();
                    state.record_guess(player_id, lat, lng);

                    // Check if all lobby members have now guessed
                    state.all_players_guessed(player_count)
                };

                if all_guessed {
                    info!("All players guessed — ending round early");
                    self.round_notify.lock().unwrap().notify_one();
                }
            }
        }
    }

    pub fn handle_lobby_event(self: &Arc<Self>, player_id: Uuid, event: LobbyUserEvent) {
        match event {
            LobbyUserEvent::Ready => {
                self.lobby.lock().unwrap().player_ready(&player_id);
                let _ = self.broadcast.send(GeoGuessrServerEvent::LobbyEvent(
                    LobbyServerEvent::PlayerReady {
                        player_id: player_id.clone(),
                    },
                ));
                if self.lobby.lock().unwrap().all_ready()
                    && self.lobby.lock().unwrap().status == LobbyStatus::Waiting
                {
                    let _ = self.broadcast.send(GeoGuessrServerEvent::GameEvent(
                        GeoGuessrGameEvent::AllReady,
                    ));
                    let l = Arc::clone(self);
                    tokio::spawn(async move {
                        let res = l.load_locations().await;
                        info!("Locations loaded: {:?}", l.state.lock().unwrap().locations);
                        match res {
                            Ok(_) => {
                                l.update_lobby_status(LobbyStatus::Playing);
                            }
                            Err(e) => {
                                l.update_lobby_status(LobbyStatus::Waiting);
                                l.lobby.lock().unwrap().player_unready(&player_id);
                                let _ = l.broadcast.send(GeoGuessrServerEvent::LobbyEvent(
                                    LobbyServerEvent::PlayerUnready { player_id },
                                ));
                                let _ = l.broadcast.send(GeoGuessrServerEvent::GameEvent(
                                    GeoGuessrGameEvent::LoadingError {
                                        message: format!("Failed to load locations: {}", e),
                                    },
                                ));
                                return;
                            }
                        }
                        GeoGuessr::run_game(l).await;
                    });
                }
            }
            LobbyUserEvent::Unready => {
                self.lobby.lock().unwrap().player_unready(&player_id);
                let _ = self.broadcast.send(GeoGuessrServerEvent::LobbyEvent(
                    LobbyServerEvent::PlayerUnready {
                        player_id: player_id.clone(),
                    },
                ));
            }
            _ => {
                return;
            }
        }
    }

    pub async fn run_game(game: Arc<GeoGuessr>) {
        info!("Starting GeoGuessr game");
        let settings = {
            let s = game.settings.lock().unwrap();
            s.clone()
        };
        let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(
            GeoGuessrGameEvent::GameStart,
        ));

        for _ in 0..settings.num_rounds {
            if game.lobby.lock().unwrap().empty() {
                info!("Game empty, terminating loop");
                return;
            }

            let location = {
                let mut state = game.state.lock().unwrap();
                match state.get_next_location() {
                    Some(s) => s,
                    None => {
                        info!("No locations left, ending game");
                        let _ = game
                            .broadcast
                            .send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::GameEnd));
                        break;
                    }
                }
            };

            let round_notify = Arc::new(Notify::new());
            *game.round_notify.lock().unwrap() = Arc::clone(&round_notify);

            game.state.lock().unwrap().begin_round();

            info!(location=%location.image_id, "ROUNDSTART");
            let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(
                GeoGuessrGameEvent::RoundStart {
                    image_id: location.image_id.clone(),
                },
            ));

            tokio::select! {
                _ = sleep(Duration::from_secs(settings.round_length_seconds as u64)) => {
                    info!("ROUNDEND (timeout)");
                }
                _ = round_notify.notified() => {
                    info!("ROUNDEND (all players guessed)");
                }
            }

            let round_results = {
                let mut state = game.state.lock().unwrap();
                let player_ids: Vec<Uuid> = state.scores.keys().cloned().collect();
                let results = state.build_round_results(&location, &player_ids); // no deadlock — same lock
                state.calculate_and_apply_scores(&location, &player_ids);
                results
            };

            let _ = game.broadcast.send(GeoGuessrServerEvent::GameEvent(
                GeoGuessrGameEvent::RoundEnd {
                    correct_lat: location.lat,
                    correct_lng: location.lng,
                    leaderboard: game.get_leaderboard(),
                    results: round_results,
                },
            ));

            info!("{:?}", settings.round_delay_seconds);
            sleep(Duration::from_secs(settings.round_delay_seconds as u64)).await;
        }

        info!("GAME END");
        if (settings.round_delay_seconds as u64) < 3 {
            sleep(Duration::from_secs(3 - settings.round_delay_seconds as u64)).await;
        }
        let _ = game
            .broadcast
            .send(GeoGuessrServerEvent::GameEvent(GeoGuessrGameEvent::GameEnd));
        game.reset();
    }
}

impl ConnectionManager for GeoGuessr {
    fn connection_drop(&self, player_id: Uuid) {
        self.lobby.lock().unwrap().player_leave(&player_id);
        self.state.lock().unwrap().scores.remove(&player_id);
        info!(
            "Player {} disconnected from lobby: {}",
            player_id, self.lobby_code
        );
        let _ = self.broadcast.send(GeoGuessrServerEvent::LobbyEvent(
            LobbyServerEvent::PlayerLeave {
                player_id: player_id.clone(),
            },
        ));
    }
    fn lobby_code(&self) -> String {
        return self.lobby_code.clone();
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
    pub map: String,
    pub map_center: (f32, f32),
}

impl GeoGuessrSettings {
    pub fn new() -> Self {
        Self {
            num_rounds: 10,
            round_length_seconds: 30,
            round_delay_seconds: 5,
            map: "World".to_string(),
            map_center: (0.0,0.0),
        }
    }

    pub fn update_game_settings(&mut self, settings: GeoGuessrSettings) {
        self.num_rounds = settings.num_rounds;
        self.round_length_seconds = settings.round_length_seconds;
        self.round_delay_seconds = settings.round_delay_seconds;
        self.map = settings.map.clone();
        self.map_center = MAPS
        .get(&settings.map)
        .map(|m| m.center)
        .unwrap_or((67.0,0.0));
        info!("{:?}", self.map_center);
    }
}

/// ===============================================
/// State
/// ===============================================
pub(crate) struct GeoGuessrState {
    pub scores: HashMap<Uuid, u32>,
    // Current round's guesses: player_id -> (lat, lng)
    pub current_round_guesses: HashMap<Uuid, (f32, f32)>,
    // All rounds' guesses (accumulated for history/replay if needed)
    pub guesses: Vec<HashMap<Uuid, (f32, f32)>>,
    pub locations: Vec<Location>,
    pub location_index: usize,
}

impl GeoGuessrState {
    pub fn new() -> Self {
        GeoGuessrState {
            scores: HashMap::new(),
            current_round_guesses: HashMap::new(),
            guesses: Vec::new(),
            locations: Vec::new(),
            location_index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.scores.iter_mut().for_each(|(_, score)| *score = 0);
        self.location_index = 0;
        self.locations = Vec::new();
        self.current_round_guesses.clear();
        self.guesses.clear();
    }

    pub fn begin_round(&mut self) {
        self.current_round_guesses.clear();
    }

    pub fn record_guess(&mut self, player_id: Uuid, lat: f32, lng: f32) {
        self.current_round_guesses
            .entry(player_id)
            .or_insert((lat, lng));
    }

    pub fn all_players_guessed(&self, player_count: usize) -> bool {
        player_count > 0 && self.current_round_guesses.len() >= player_count
    }

    /// Scores all players for the current round using Haversine distance,
    /// then saves the round's guesses into history.
    pub fn calculate_and_apply_scores(&mut self, correct: &Location, player_ids: &[Uuid]) {
        for player_id in player_ids {
            let points = match self.current_round_guesses.get(player_id) {
                Some(&(lat, lng)) => haversine_score(lat, lng, correct.lat, correct.lng),
                // No guess submitted — 0 points
                None => 0,
            };
            self.increment_player_score(player_id, points);
        }
        self.guesses.push(self.current_round_guesses.clone());
    }

    pub fn build_round_results(
        &self,
        correct: &Location,
        player_ids: &[Uuid],
    ) -> HashMap<Uuid, PlayerRoundResult> {
        player_ids
            .iter()
            .map(|id| {
                let result = match self.current_round_guesses.get(id) {
                    Some(&(lat, lng)) => {
                        let distance_km = haversine_km(lat, lng, correct.lat, correct.lng) as f32;
                        let points_gained = haversine_score(lat, lng, correct.lat, correct.lng);
                        PlayerRoundResult {
                            guess: Some((lat, lng)),
                            distance_km: Some(distance_km),
                            points_gained,
                        }
                    }
                    None => PlayerRoundResult {
                        guess: None,
                        distance_km: None,
                        points_gained: 0,
                    },
                };
                (*id, result)
            })
            .collect()
    }

    pub fn get_current_location(&self) -> Option<Location> {
        if self.locations.is_empty()
            || self.location_index == 0
            || self.location_index - 1 >= self.locations.len()
        {
            return None;
        }
        Some(Location {
            image_id: self.locations[self.location_index - 1].image_id.clone(),
            lat: self.locations[self.location_index - 1].lat,
            lng: self.locations[self.location_index - 1].lng,
        })
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

fn haversine_km(lat1: f32, lng1: f32, lat2: f32, lng2: f32) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    let (lat1, lng1, lat2, lng2) = (lat1 as f64, lng1 as f64, lat2 as f64, lng2 as f64);
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_KM * c
}

fn haversine_score(lat1: f32, lng1: f32, lat2: f32, lng2: f32) -> u32 {
    let distance_km = haversine_km(lat1, lng1, lat2, lng2);
    let score = 5000.0 * (-distance_km / 2000.0_f64).exp();
    score.round() as u32
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
        results: HashMap<Uuid, PlayerRoundResult>,
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
    UpdateGameSettings { settings: GeoGuessrSettings },
    Guess { lat: f32, lng: f32 },
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
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Location {
    pub image_id: String,
    pub lat: f32,
    pub lng: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerRoundResult {
    pub guess: Option<(f32, f32)>,
    pub distance_km: Option<f32>,
    pub points_gained: u32,
}
