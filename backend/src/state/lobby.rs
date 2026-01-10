use std::collections::HashMap;

use crate::state::LobbyStatus;

#[derive(Debug)]
pub(crate) struct LobbyState {
    pub players: HashMap<String, (String, bool)>,
    pub status: LobbyStatus,
}

impl LobbyState {
    pub fn new() -> Self {
        LobbyState {
            players: HashMap::new(),
            status: LobbyStatus::Waiting,
        }
    }
}

impl LobbyState {
    pub fn player_join(&mut self, player_id: String, player_username: String) {
        self.players.insert(player_id, (player_username, false));
    }

    pub fn player_ready(&mut self, user_id: &str) {
        if let Some((_username, ready)) = self.players.get_mut(user_id) {
            *ready = true;
        }
    }

    pub fn get_players(&self) -> Vec<(String, String, bool)> {
        self.players
            .iter()
            .map(|(id, (username, ready))| (id.clone(), username.clone(), *ready))
            .collect()
    }

    pub fn all_ready(&self) -> bool {
        self.players.values().all(|(_, ready)| *ready)
    }

    pub fn player_leave(&mut self, player_id: &str) {
        self.players.remove(player_id);
    }

    pub fn update_lobby_status(&mut self, new_status: LobbyStatus) {
        self.status = new_status;
    }
}
