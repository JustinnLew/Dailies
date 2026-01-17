use rspotify::ClientCredsSpotify;
use std::sync::Arc;
use tokio::sync::mpsc;

pub mod games;
pub mod guessthesong;
pub mod lobby;

pub(crate) use games::*;
pub(crate) use guessthesong::*;
pub(crate) use lobby::*;

#[derive(Clone)]
pub(crate) struct AppState {
    pub games: Arc<Games>,
    pub spotify_client: Arc<ClientCredsSpotify>,
    pub cleanup: mpsc::UnboundedSender<String>,
}

impl AppState {
    pub fn new(spotify: ClientCredsSpotify, cleanup: mpsc::UnboundedSender<String>) -> Self {
        AppState {
            games: Arc::new(Games::new()),
            spotify_client: Arc::new(spotify),
            cleanup: cleanup,
        }
    }
}
