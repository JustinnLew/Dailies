use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast::{self};

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "action", content = "data")]
pub(crate) enum ServerEvent {
    SyncState {
        players: Vec<(String, String, bool)>,
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
}

#[derive(Debug)]
pub(crate) struct Song {
    title: String,
    artists: Vec<String>,
    url: String,
}

#[derive(Debug)]
pub(crate) enum GameState {
    GuessTheSong {
        scores: HashMap<String, u32>,
        songs: Vec<Song>,
        chat: Vec<(String, String)>,
        round_length_seconds: u8,
    },
}

#[derive(Debug)]
pub(crate) enum LobbyStatus {
    Waiting,
    Playing,
    Finished,
}

#[derive(Debug)]
pub(crate) struct LobbyState {
    players: HashMap<String, (String, bool)>,
    status: LobbyStatus,
    game: GameState,
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
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub games: Arc<Games>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            games: Arc::new(Games::new()),
        }
    }
}
