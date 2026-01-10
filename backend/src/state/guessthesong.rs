use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use rand::seq::SliceRandom;

use crate::state::{LobbyState};

/// ===============================================
/// Main Parent Struct for Guess The Song Game
/// ===============================================
pub(crate) struct GuessTheSongGame {
    pub lobby_state: Mutex<LobbyState>,
    pub broadcast: broadcast::Sender<GuessTheSongServerEvent>,
    pub settings: Mutex<GuessTheSongGameSettings>,
    pub state: Mutex<GuessTheSongGameState>,
}

impl GuessTheSongGame {
    pub fn player_join(&self, player_id: String, player_username: String) {
        let mut lobby = self.lobby_state.lock().unwrap();
        lobby.player_join(player_id, player_username);
    }

    pub fn player_ready(&self, user_id: &str) {
        let mut lobby = self.lobby_state.lock().unwrap();
        lobby.player_ready(user_id);
    }

    pub fn all_ready(&self) -> bool {
        self.lobby_state.lock().unwrap().all_ready()
    }

    pub fn update_lobby_status(&self, new_status: crate::state::LobbyStatus) {
        let mut lobby = self.lobby_state.lock().unwrap();
        lobby.update_lobby_status(new_status);
    }

    pub fn get_players(&self) -> Vec<(String, String, bool)> {
        self.lobby_state.lock().unwrap().get_players()
    }

    pub fn get_num_songs(&self) -> u8 {
        self.settings.lock().unwrap().get_num_songs()
    }

    pub fn get_playlist_link(&self) -> String {
        self.settings.lock().unwrap().get_playlist_link().to_string()
    }

    pub fn update_game_settings(&self, settings: GuessTheSongGameSettings) {
        let mut game_settings = self.settings.lock().unwrap();
        game_settings.update_game_settings(settings);
    }
}

/// ===============================================
/// Settings
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GuessTheSongGameSettings {
    pub playlist_link: String,
    pub num_songs: u8,
}

impl GuessTheSongGameSettings {
    pub fn new() -> Self {
        Self {
            playlist_link: String::new(),
            num_songs: 10,
        }
    }

    pub fn get_playlist_link(&self) -> &String {
        &self.playlist_link
    }

    pub fn get_num_songs(&self) -> u8 {
        self.num_songs
    }

    pub fn update_game_settings(&mut self, settings: GuessTheSongGameSettings) {
        self.num_songs = settings.num_songs;
        self.playlist_link = settings.playlist_link;
    }
}

/// ===============================================
/// State
/// ===============================================
pub(crate) struct GuessTheSongGameState {
    pub scores: HashMap<String, u32>,
    pub songs: Vec<Song>,
    pub chat: Vec<(String, String)>,
    pub round_length_seconds: u8,
}

impl GuessTheSongGameState {
    pub fn new() -> Self {
        GuessTheSongGameState {
            scores: HashMap::new(),
            songs: Vec::new(),
            chat: Vec::new(),
            round_length_seconds: 30,
        }
    }

    pub fn add_song(&mut self, song: Song) {
        self.songs.push(song);
    }

    pub fn shuffle_songs(&mut self) {
        let mut rng = rand::rng();
        self.songs.shuffle(&mut rng);
    }
}

/// ===============================================
/// Server Events
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event", content = "data")]
pub(crate) enum GuessTheSongServerEvent {
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
        settings: GuessTheSongGameSettings,
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

/// ===============================================
/// Helper Structs
/// ===============================================

#[derive(Debug)]
pub(crate) struct Song {
    pub title: String,
    pub artists: Vec<String>,
    pub url: String,
}