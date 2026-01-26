use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::state::{LobbyState, LobbyStatus};

/// ===============================================
/// Main Parent Struct for Guess The Song Game
/// ===============================================
pub(crate) struct GuessTheSongGame {
    pub lobby_state: Mutex<LobbyState>,
    pub broadcast: broadcast::Sender<GuessTheSongServerEvent>,
    pub settings: Mutex<GuessTheSongGameSettings>,
    pub state: Mutex<GuessTheSongGameState>,
    pub lobby_code: String,
}

pub(crate) enum PlayerJoinResult {
    ReJoin,
    NewJoin,
}

impl GuessTheSongGame {
    pub fn reset(&self) {
        self.lobby_state.lock().unwrap().reset();
        self.state.lock().unwrap().reset();
    }

    pub fn get_new_player_id(&self) -> Uuid {
        loop {
            let id = Uuid::new_v4();
            if !self.lobby_state.lock().unwrap().players.contains_key(&id) {
                return id;
            }
        }
    }

    pub fn player_join(
        &self,
        player_id: Uuid,
        player_username: String,
    ) -> Result<PlayerJoinResult, &str> {
        let mut lobby = self.lobby_state.lock().unwrap();
        let mut state = self.state.lock().unwrap();
        if state.scores.contains_key(&player_id) {
            lobby.player_join(player_id.clone(), player_username);
            return Ok(PlayerJoinResult::ReJoin);
        } else {
            if lobby.status != crate::state::LobbyStatus::Waiting {
                return Err("Cannot join game in progress");
            }
            lobby.player_join(player_id.clone(), player_username);
            state.scores.insert(player_id.clone(), 0);
        }
        Ok(PlayerJoinResult::NewJoin)
    }

    pub fn player_ready(&self, user_id: &Uuid) {
        self.lobby_state.lock().unwrap().player_ready(user_id);
    }

    pub fn player_unready(&self, user_id: &Uuid) {
        self.lobby_state.lock().unwrap().player_unready(user_id);
    }

    pub fn all_ready(&self) -> bool {
        self.lobby_state.lock().unwrap().all_ready()
    }

    pub fn get_lobby_status(&self) -> LobbyStatus {
        self.lobby_state.lock().unwrap().status.clone()
    }

    pub fn update_lobby_status(&self, new_status: LobbyStatus) {
        let mut lobby = self.lobby_state.lock().unwrap();
        lobby.update_lobby_status(new_status);
    }

    pub fn get_players(&self) -> Vec<(Uuid, String, bool)> {
        self.lobby_state.lock().unwrap().get_players()
    }

    pub fn get_num_songs(&self) -> u8 {
        self.settings.lock().unwrap().get_num_songs()
    }

    pub fn get_playlist_link(&self) -> String {
        self.settings
            .lock()
            .unwrap()
            .get_playlist_link()
            .to_string()
    }

    pub fn get_current_song(&self) -> Option<Song> {
        self.state.lock().unwrap().get_current_song()
    }

    pub fn update_game_settings(&self, settings: GuessTheSongGameSettings) {
        let mut game_settings = self.settings.lock().unwrap();
        game_settings.update_game_settings(settings);
    }

    pub fn is_correct_artist(&self, guess: &str) -> Option<String> {
        self.state.lock().unwrap().is_correct_artist(guess)
    }

    pub fn is_correct_song(&self, guess: &str) -> Option<String> {
        self.state.lock().unwrap().is_correct_song(guess)
    }

    pub fn get_leaderboard(&self) -> HashMap<Uuid, u32> {
        let state = self.state.lock().unwrap();
        state.scores.clone()
    }

    pub fn increment_player_score(&self, player_id: &Uuid, points: u32) {
        self.state
            .lock()
            .unwrap()
            .increment_player_score(player_id, points);
    }

    pub fn get_settings(&self) -> GuessTheSongGameSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn get_round_start_time(&self) -> Option<u64> {
        self.state.lock().unwrap().get_round_start_time()
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
    pub round_length_seconds: u8,
    pub answer_delay_seconds: u8,
    pub round_delay_seconds: u8,
}

impl GuessTheSongGameSettings {
    pub fn new() -> Self {
        Self {
            playlist_link: String::new(),
            num_songs: 10,
            round_length_seconds: 30,
            answer_delay_seconds: 5,
            round_delay_seconds: 3,
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
        self.round_length_seconds = settings.round_length_seconds;
        self.answer_delay_seconds = settings.answer_delay_seconds;
        self.round_delay_seconds = settings.round_delay_seconds;
    }
}

/// ===============================================
/// State
/// ===============================================
pub(crate) struct GuessTheSongGameState {
    pub scores: HashMap<Uuid, u32>,
    pub songs: Vec<SongState>,
    pub song_index: usize,
    pub round_start_time: Option<u64>,
}

impl GuessTheSongGameState {
    pub fn new() -> Self {
        GuessTheSongGameState {
            scores: HashMap::new(),
            songs: Vec::new(),
            song_index: 0,
            round_start_time: None,
        }
    }

    pub fn reset(&mut self) {
        self.scores.iter_mut().for_each(|(_, score)| *score = 0);
        self.song_index = 0;
        self.songs = Vec::new();
        self.round_start_time = None;
    }

    pub fn get_round_start_time(&self) -> Option<u64> {
        self.round_start_time
    }

    pub fn get_current_song(&self) -> Option<Song> {
        if self.songs.is_empty() || self.song_index == 0 || self.song_index - 1 >= self.songs.len()
        {
            return None;
        }
        Some(Song {
            title: self.songs[self.song_index - 1].title.0.clone(),
            artists: self.songs[self.song_index - 1]
                .artists
                .iter()
                .map(|(artist, _)| artist)
                .cloned()
                .collect(),
            url: self.songs[self.song_index - 1].url.clone(),
        })
    }

    pub fn add_song(&mut self, song: SongState) {
        self.songs.push(song);
    }

    pub fn get_next_song(&mut self) -> Option<Song> {
        self.song_index += 1;
        self.get_current_song()
    }

    pub fn is_correct_artist(&mut self, guess: &str) -> Option<String> {
        let guess = guess.trim().to_lowercase();
        for (artist, found) in self.songs[self.song_index - 1].artists.iter_mut() {
            if !*found && artist.trim().to_lowercase() == guess {
                *found = true;
                return Some(artist.clone());
            }
        }
        None
    }

    pub fn is_correct_song(&mut self, guess: &str) -> Option<String> {
        if self.song_index == 0 || self.songs.is_empty() {
            return None;
        }
        if !self.songs[self.song_index - 1].title.1
            && self.songs[self.song_index - 1]
                .title
                .0
                .trim()
                .to_lowercase()
                == guess.trim().to_lowercase()
        {
            self.songs[self.song_index - 1].title.1 = true;
            return Some(self.songs[self.song_index - 1].title.0.clone());
        }
        None
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
pub(crate) enum GuessTheSongServerEvent {
    SyncState {
        players: Vec<(Uuid, String, bool)>,
        settings: GuessTheSongGameSettings,
        leaderboard: HashMap<Uuid, u32>,
        preview_url: Option<String>,
        status: LobbyStatus,
        round_start_time: Option<u64>,
    },
    PlayerJoin {
        player_id: Uuid,
        player_username: String,
    },
    PlayerReady {
        player_id: Uuid,
    },
    PlayerUnready {
        player_id: Uuid,
    },
    PlayerLeave {
        player_id: Uuid,
    },
    AllReady,
    GameStart,
    GameSettingsUpdated {
        settings: GuessTheSongGameSettings,
    },
    RoundStart {
        preview_url: String,
        round_start_time: u64,
    },
    RoundEnd {
        correct_title: String,
        correct_artists: Vec<String>,
        leaderboard: HashMap<Uuid, u32>,
    },
    GameEnd,
    PlayerGuess {
        username: String,
        content: String,
    },
    CorrectGuess {
        player_id: Uuid,
        msg: String,
    },
    JoinError {
        message: String,
    },
    PlaylistError {
        message: String,
    },
}

/// ===============================================
/// User Events
/// ===============================================
#[derive(Deserialize, Debug)]
#[serde(tag = "event")]
pub(crate) enum GuessTheSongUserEvent {
    Join {
        lobby_code: String,
        username: String,
    },
    Ready,
    Unready,
    UpdateGameSettings {
        settings: GuessTheSongGameSettings,
    },
    Guess {
        content: String,
    },
}
/// ===============================================
/// Helper Structs
/// ===============================================

#[derive(Debug)]
pub(crate) struct SongState {
    pub title: (String, bool),
    pub artists: Vec<(String, bool)>,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Song {
    pub title: String,
    pub artists: Vec<String>,
    pub url: String,
}
