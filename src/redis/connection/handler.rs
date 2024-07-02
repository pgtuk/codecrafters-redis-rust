use std::sync::Arc;

use tokio::sync::broadcast::Sender;

use crate::redis::{cmd, ServerInfo};
use crate::redis::cmd::Command;
use crate::redis::connection::Connection;
use crate::redis::db::Db;
use crate::redis::frame::Frame;

pub struct Handler {
    connection: Connection,
    db: Db,
    server_info: ServerInfo,
    sender: Arc<Sender<Frame>>,
}

impl Handler {
    pub(crate) fn new(
        connection: Connection,
        db: Db,
        server_info: ServerInfo,
        sender: Arc<Sender<Frame>>,
    ) -> Handler {
        Handler {
            connection,
            db,
            server_info,
            sender,
        }
    }

    pub async fn handle_connection(&mut self) -> anyhow::Result<()> {
        loop {
            let opt_frame = self.connection.read_frame().await?;
            let frame = match opt_frame {
                Some(frame) => { frame }
                // None means that the socket was closed by peer
                None => return Ok(()),
            };

            match self.run_as_cmd(&frame).await {
                Ok(cmd) => {
                    if self.server_info.is_master() {
                        match cmd {
                            // replicate write commands
                            Command::Set(_) => { self.sender.send(frame)?; }

                            // after psync cmd master starts listening for write commands to replicate
                            Command::Psync(_) => {
                                let mut receiver = self.sender.subscribe();

                                while let Ok(frame) = receiver.recv().await {
                                    dbg!(&frame);
                                    self.connection.write_frame(&frame).await?
                                }
                            }
                            _ => (),
                        }
                    };
                }
                _ => continue
            }
        }
    }

    async fn run_as_cmd(&mut self, frame: &Frame) -> anyhow::Result<Command> {
        let cmd = cmd::Command::from_frame(frame)?;
        cmd.apply(
            &mut self.connection,
            &mut self.db,
            &self.server_info,
        ).await?;

        Ok(cmd)
    }
}
