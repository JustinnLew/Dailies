use rspotify::ClientCredsSpotify;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast::{self};

use crate::guess_the_song::GuessTheSongGameSettings;

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "event", content = "data")]
pub(crate) enum ServerEvent {
    SyncState {
        players: Vec<(String, String, bool)>,
        num_songs: u8,
        playlist_link: String,
    },
    PlayerJoin {
        player_id: String,
        player_username: String,
    },
    PlayerReady {
        player_id: String,
    },
    PlayerLeave {
        player_id: String,
    },
    AllReady,
    GameSettingsUpdated {
        settings: GameSettings,
    },
    RoundStart {
        preview_url: String,
    },
    RoundEnd {
        correct_title: String,
        correct_artists: Vec<String>,
    },
    GameEnd,
}

#[derive(Debug)]
pub(crate) struct Song {
    pub title: String,
    pub artists: Vec<String>,
    pub url: String,
}

#[derive(Debug)]
pub(crate) enum GameState {
    GuessTheSong {
        scores: HashMap<String, u32>,
        songs: Vec<Song>,
        chat: Vec<(String, String)>,
        round_length_seconds: u8,
    },
    AAA {},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum GameSettings {
    GuessTheSong(GuessTheSongGameSettings),
}

#[derive(Debug, PartialEq)]
pub(crate) enum LobbyStatus {
    Waiting,
    Playing,
}

#[derive(Debug)]
pub(crate) struct LobbyState {
    pub players: HashMap<String, (String, bool)>,
    pub status: LobbyStatus,
    pub game: GameState,
    pub settings: GameSettings,
}

#[derive(Debug)]
pub(crate) struct Lobby {
    pub state: Mutex<LobbyState>,
    pub broadcast: broadcast::Sender<ServerEvent>,
}

pub(crate) struct Games {
    pub games: Mutex<HashMap<String, Arc<Lobby>>>,
}

impl Games {
    pub fn new() -> Self {
        Games {
            games: Mutex::new(HashMap::new()),
        }
    }

    // For future: Need to change for other games
    pub fn add_lobby(&self, lobby_code: &String) {
        let (send, _) = broadcast::channel::<ServerEvent>(64);
        let lobby = Lobby {
            state: Mutex::new(LobbyState {
                players: HashMap::new(),
                status: LobbyStatus::Waiting,
                game: GameState::GuessTheSong {
                    scores: HashMap::new(),
                    songs: Vec::new(),
                    chat: Vec::new(),
                    round_length_seconds: 30,
                },
                settings: GameSettings::GuessTheSong(GuessTheSongGameSettings::new()),
            }),
            broadcast: send,
        };
        self.games
            .lock()
            .unwrap()
            .insert(lobby_code.to_string(), Arc::new(lobby));
    }

    pub fn get_lobby(&self, lobby_code: &String) -> Option<Arc<Lobby>> {
        let games = self.games.lock().unwrap();
        games.get(lobby_code).cloned()
    }
}

impl Lobby {
    pub fn player_join(&self, player_id: String, player_username: String) {
        let mut state = self.state.lock().unwrap();
        state.players.insert(player_id, (player_username, false));
    }

    pub fn player_ready(&self, user_id: &String) {
        let mut state = self.state.lock().unwrap();
        if let Some((_username, ready)) = state.players.get_mut(user_id) {
            *ready = true;
        }
    }

    pub fn get_players(&self) -> Vec<(String, String, bool)> {
        let state = self.state.lock().unwrap();
        state
            .players
            .iter()
            .map(|(id, (username, ready))| (id.clone(), username.clone(), *ready))
            .collect()
    }

    pub fn all_ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.players.values().all(|(_, ready)| *ready)
    }

    pub fn player_leave(&self, player_id: &String) {
        let mut state = self.state.lock().unwrap();
        state.players.remove(player_id);
    }

    pub fn update_game_settings(&self, settings: &GameSettings) {
        let mut state = self.state.lock().unwrap();
        state.settings = settings.clone();
    }

    pub fn get_game_settings(&self) -> GameSettings {
        let state = self.state.lock().unwrap();
        state.settings.clone()
    }

    pub fn update_lobby_status(&self, status: LobbyStatus) {
        let mut state = self.state.lock().unwrap();
        state.status = status;
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub games: Arc<Games>,
    pub spotify_client: Arc<ClientCredsSpotify>,
}

impl AppState {
    pub fn new(spotify: ClientCredsSpotify) -> Self {
        AppState {
            games: Arc::new(Games::new()),
            spotify_client: Arc::new(spotify),
        }
    }
}
