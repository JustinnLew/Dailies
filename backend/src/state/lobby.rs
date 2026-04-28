use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LobbyStatus {
    Waiting,
    Loading,
    Playing,
    Finished,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(tag = "event")]
pub(crate) enum LobbyServerEvent {
    PlayerJoin { player_id: Uuid, player_username: String },
    PlayerLeave { player_id: Uuid },
    PlayerReady { player_id: Uuid },
    PlayerUnready { player_id: Uuid },
    UpdateLobbyStatus { new_status: LobbyStatus },
    JoinError { message: String },
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(tag = "event")]
pub(crate) enum LobbyUserEvent {
    Join { lobby_code: String, username: String },
    Ready,
    Unready,
}

#[derive(Debug)]
pub(crate) struct LobbyState {
    pub players: HashMap<Uuid, (String, bool)>,
    pub status: LobbyStatus,
}

impl LobbyState {
    pub fn new() -> Self {
        LobbyState {
            players: HashMap::new(),
            status: LobbyStatus::Waiting,
        }
    }

    pub fn reset(&mut self) {
        self.status = LobbyStatus::Waiting;
        self.players.iter_mut().for_each(|(_, v)| v.1 = false);
    }

    pub fn player_join(&mut self, player_id: Uuid, player_username: String) {
        self.players.insert(player_id, (player_username, false));
    }

    pub fn player_ready(&mut self, user_id: &Uuid) {
        if let Some((_, ready)) = self.players.get_mut(user_id) {
            *ready = true;
        }
    }

    pub fn player_unready(&mut self, user_id: &Uuid) {
        if let Some((_, ready)) = self.players.get_mut(user_id) {
            *ready = false;
        }
    }

    pub fn get_players(&self) -> Vec<(Uuid, String, bool)> {
        self.players
            .iter()
            .map(|(id, (username, ready))| (id.clone(), username.clone(), *ready))
            .collect()
    }

    pub fn all_ready(&self) -> bool {
        self.players.values().all(|(_, ready)| *ready)
    }

    pub fn player_leave(&mut self, player_id: &Uuid) {
        self.players.remove(player_id);
    }

    pub fn update_lobby_status(&mut self, new_status: LobbyStatus) {
        self.status = new_status;
    }

    pub fn empty(&self) -> bool {
        self.players.is_empty()
    }

    pub fn get_new_player_id(&self) -> Uuid {
        loop {
            let new_id = Uuid::new_v4();
            if !self.players.contains_key(&new_id) {
                return new_id;
            }
        }
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}
