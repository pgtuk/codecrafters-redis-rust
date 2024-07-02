use std::sync::Arc;

use tokio::sync::broadcast;

use crate::redis::{cmd, Role, ServerInfo};
use crate::redis::cmd::Command;
use crate::redis::db::Db;
use crate::redis::frame::Frame;

use super::Connection;

//      mpsc::Sender<Message>
pub struct Handler {
    connection: Connection,
    db: Db,
    server_info: ServerInfo,
    sender: Arc<broadcast::Sender<Frame>>,
}

impl Handler {
    pub(crate) fn new(
        connection: Connection,
        db: Db,
        server_info: ServerInfo,
        sender: Arc<broadcast::Sender<Frame>>

    ) -> Handler {
        Handler {
            connection,
            db,
            server_info,
            sender,
        }
    }

    fn is_master(&self) -> bool {
        self.server_info.role == Role::Master
    }

    async fn handle_connection(&mut self) -> anyhow::Result<()> {
        loop {
            let opt_frame =  self.connection.read_frame().await?;

            let frame = match opt_frame {
                Some(frame) => {frame},
                // None means that the socket was closed by peer
                None => return Ok(()),
            };

            let cmd = cmd::Command::from_frame(&frame)?;

            cmd.apply(
                &mut self.connection,
                &mut self.db,
                &self.server_info
            ).await?;

            // if let Command::Set(set) = cmd {
            //     self.sender.send(frame)?;
            // }

            match cmd {
                Command::Set(_) => if self.is_master() {self.sender.send(frame)?;},
                Command::Psync(_) => {},
                _ => ()
            }

        }
    }
}
