use tokio::sync::{mpsc, oneshot, broadcast};

use crate::redis::{cmd, Role, ServerInfo};
use crate::redis::channels::Message;
use crate::redis::db::Db;

use super::Connection;
use tokio::time::{self, Duration};

pub struct Handler {
    connection: Connection,
    db: Db,
    server_info: ServerInfo,
    sender: mpsc::Sender<Message>,
    repl_listener: Option<broadcast::Receiver<cmd::Command>>
}

impl Handler {
    pub(crate) fn new(
        connection: Connection,
        db: Db,
        server_info: ServerInfo,
        sender: mpsc::Sender<Message>
    ) -> Handler {
        Handler {
            connection,
            db,
            server_info,
            sender,
            repl_listener: None
        }
    }

    pub(crate) async fn run(&mut self) -> anyhow::Result<()> {
        // do something with this abomination
        if self.is_master() {
            loop {
                self.handle_connection().await?
            }
        } else {
            loop {
                tokio::select! {
                    _ = sleep(100) => self.handle_connection().await?,
                    cmd = self.replication_cmd() => self.replicate(cmd?).await?,
                }
            }
        }   
    }

    async fn handle_connection(&mut self) -> anyhow::Result<()> {
        // loop {
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

            if self.server_info.role == Role::Master && cmd.is_write() {
                self.sender.send(Message::Propagate(cmd))
                .await
                .expect("Looks like channel manager is gone");
            } else if self.server_info.role == Role::Slave && cmd.is_handshake() && self.repl_listener.is_none() {
                let (sx, rx) = oneshot::channel();
                self.sender.send(Message::Handshake(sx))
                .await
                .expect("Looks like channel manager is gone");
   
                self.repl_listener = Some(rx.await?);
            };

            Ok(())
        // }
    }

    async fn replication_cmd(&mut self) -> anyhow::Result<Option<cmd::Command>> {
        match &mut self.repl_listener {
            Some(listener) => {
                Ok(Some(listener.recv().await?))
            },
            None => { 
                sleep(100).await;
                Ok(None)
            },
        }
    }

    async fn replicate(&mut self, cmd: Option<cmd::Command>) -> anyhow::Result<()> {
        if let Some(cmd) = cmd {
            cmd.apply(
                &mut self.connection, 
                &mut self.db, 
                &self.server_info,
            ).await
        } else {
            Ok(())
        }
    }

    fn is_master(&self) -> bool {
        self.server_info.role == Role::Master
    }
}

async fn sleep(milis: u64) {
    time::sleep(Duration::from_millis(milis)).await;
}