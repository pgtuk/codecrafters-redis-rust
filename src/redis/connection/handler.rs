use tokio::sync::{mpsc, oneshot};

use crate::redis::{cmd, ServerInfo};
use crate::redis::channels::Message;
use crate::redis::db::Db;

use super::Connection;

pub struct Handler {
    connection: Connection,
    db: Db,
    server_info: ServerInfo,
    tx: mpsc::Sender<Message>
}

impl Handler {
    pub(crate) fn new(
        connection: Connection,
        db: Db,
        server_info: ServerInfo,
        tx: mpsc::Sender<Message>
    ) -> Handler {
        Handler {
            connection,
            db,
            server_info,
            tx
        }
    }

    pub(crate) async fn handle_connection(&mut self) -> anyhow::Result<()> {
        loop {
            let opt_frame =  self.connection.read_frame().await?;

            let frame = match opt_frame {
                Some(frame) => {frame},
                // None means that the socket was closed by peer
                None => return Ok(()),
            };

            let cmd = cmd::Command::from_frame(frame)?;

            cmd.apply(
                &mut self.connection,
                &mut self.db,
                &self.server_info
            ).await?;

            if cmd.is_write() {
                self.tx.send(Message::Propagate(cmd)).await.expect("Looks like channel manager is gone");
            }
            let (tx, rx) = oneshot::channel();
            if cmd.is_handshake() {
                self.tx.send(Message::Handshake(tx)).await.expect("Looks like channel manager is gone");;
            }
        }
    }
}