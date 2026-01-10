use std::{sync::{Arc, Mutex}};

use dashmap::DashMap;
use tokio::sync::broadcast;

use crate::state::{GuessTheSongGame, GuessTheSongGameState, GuessTheSongServerEvent, LobbyState, guessthesong::GuessTheSongGameSettings};


pub(crate) enum GameType {
    GuessTheSong,
}
pub(crate) struct Games {
    pub guess_the_song: DashMap<String, Arc<GuessTheSongGame>>,
    pub registry: DashMap<String, GameType>,
}

impl Games {
    pub fn new() -> Self {
        Games {
            guess_the_song: DashMap::new(),
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
        };
        self.guess_the_song
            .insert(lobby_code.to_string(), Arc::new(lobby));
        self.registry.insert(lobby_code.to_string(), GameType::GuessTheSong);
    }

    pub fn valid_lobby_code(&self, lobby_code: &str) -> bool {
        self.registry.contains_key(lobby_code)
    }
}