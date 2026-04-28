use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

pub(crate) trait ConnectionManager {
    fn connection_drop(&self, player_id: Uuid);
    fn no_connections(&self) -> bool;
    fn lobby_code(&self) -> String;
}

pub(crate) struct ConnectionGuard<G>
where
    G: ConnectionManager,
{
    pub game: Arc<G>,
    pub player_id: Uuid,
    pub cleanup_tx: mpsc::UnboundedSender<String>,
}

impl<G> Drop for ConnectionGuard<G>
where
    G: ConnectionManager,
{
    fn drop(&mut self) {
        info!("Dropping connection for player {}", self.player_id);
        self.game.connection_drop(self.player_id);
        if self.game.no_connections() {
            info!("No more connections, cleaning up game");
            let _ = self.cleanup_tx.send(self.game.lobby_code());
        }
    }
}
