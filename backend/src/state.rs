use std::{collections::HashMap, sync::{Arc, Mutex}};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum ServerEvent {
    PlayerJoin { player : Player },
}

pub(crate) struct Song {
    title: String,
    artists: Vec<String>,
    url: String,
}
pub(crate) enum GameState {
    GuessTheSong {
        scores: HashMap<String, u32>,
        songs: Vec<Song>,
        chat: Vec<(String, String)>,
        round_length_seconds: u8,
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Player {
    pub user_id: String,
    pub username: String,
}
pub(crate) enum LobbyStatus {
    Waiting,
    Playing,
    Finished,
}

pub (crate) struct LobbyState {
    players: Vec<Player>,
    status: LobbyStatus,
    game: GameState,
}
pub(crate) struct Lobby {
    pub state: Mutex<LobbyState>,
    pub broadcast: broadcast::Sender<ServerEvent>
}

pub (crate) struct Games {
    pub games: Mutex<HashMap<String, Arc<Lobby>>>,
}


#[derive(Clone)]
pub(crate) struct AppState {
    pub games: Arc<Games>,
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
            players: vec![],
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
        self.games.lock().unwrap().insert(lobby_code.to_string(), Arc::new(lobby));
    }
    
    pub fn get_lobby(&self, lobby_code: &String) -> Option<Arc<Lobby>> {
        let games = self.games.lock().unwrap();
        games.get(lobby_code).cloned()
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            games: Arc::new(Games::new()),
        }
    }
}

impl Lobby {
    pub fn player_join(&self, player: Player) {
        let mut state = self.state.lock().unwrap();
        state.players.push(player);
    }

    pub fn broadcast(&self, msg: ServerEvent) {
        let _ = self.broadcast.send(msg);
    }
}