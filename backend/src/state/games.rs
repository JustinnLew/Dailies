use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use tokio::sync::{Notify, broadcast};

use crate::state::{
    GuessTheSongGame, GuessTheSongGameState, GuessTheSongServerEvent, LobbyState, geoguessr::{GeoGuessr, GeoGuessrServerEvent, GeoGuessrSettings, GeoGuessrState}, guessthesong::GuessTheSongGameSettings
};

pub(crate) enum GameType {
    GuessTheSong,
    GeoGuessr,
}

pub(crate) struct Games {
    pub guess_the_song: DashMap<String, Arc<GuessTheSongGame>>,
    pub geo_guessr: DashMap<String, Arc<GeoGuessr>>,
    pub registry: DashMap<String, GameType>,
}

impl Games {
    pub fn new() -> Self {
        Games {
            guess_the_song: DashMap::new(),
            geo_guessr: DashMap::new(),
            registry: DashMap::new(),
        }
    }

    pub fn add_guess_the_song_lobby(&self, lobby_code: &String) {
        let (send, _) = broadcast::channel::<GuessTheSongServerEvent>(64);
        let lobby = GuessTheSongGame {
            lobby_state: Mutex::new(LobbyState::new()),
            broadcast: send,
            settings: Mutex::new(GuessTheSongGameSettings::new()),
            state: Mutex::new(GuessTheSongGameState::new()),
            lobby_code: lobby_code.to_string(),
        };
        self.guess_the_song
            .insert(lobby_code.to_string(), Arc::new(lobby));
        self.registry
            .insert(lobby_code.to_string(), GameType::GuessTheSong);
    }

    pub fn add_geo_guessr_lobby(&self, lobby_code: &String) {
        let (send, _) = broadcast::channel::<GeoGuessrServerEvent>(64);
        let lobby = GeoGuessr {
            lobby: Mutex::new(LobbyState::new()),
            broadcast: send,
            settings: Mutex::new(GeoGuessrSettings::new()),
            state: Mutex::new(GeoGuessrState::new()),
            lobby_code: lobby_code.to_string(),
            round_notify: Mutex::new(Arc::new(Notify::new())),
        };
        self.geo_guessr
            .insert(lobby_code.to_string(), Arc::new(lobby));
        self.registry
            .insert(lobby_code.to_string(), GameType::GeoGuessr);
    }

    pub fn remove_lobby(&self, lobby_code: &str) {
        if let Some(game_type) = self.registry.get(lobby_code) {
            match *game_type {
                GameType::GuessTheSong => {
                    self.guess_the_song.remove(lobby_code);
                }
                GameType::GeoGuessr => {
                    self.geo_guessr.remove(lobby_code);
                }
            }
        }
    }

    pub fn valid_lobby_code(&self, lobby_code: &str) -> bool {
        self.registry.contains_key(lobby_code)
    }
}
