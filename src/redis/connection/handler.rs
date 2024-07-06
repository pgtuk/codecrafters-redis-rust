use std::sync::Arc;

use tokio::sync::broadcast::Sender;

use crate::redis::cmd::Command;
use crate::redis::connection::Connection;
use crate::redis::db::Db;
use crate::redis::frame::Frame;
use crate::redis::ServerInfo;

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

            let cmd = self.run_as_cmd(&frame).await?;

            self.increase_offset(frame.byte_len());

            if self.server_info.is_master() {
                match cmd {
                    // replicate write commands
                    Command::Set(_) => { self.sender.send(frame)?; }

                    // after psync cmd master starts listening for write commands to replicate
                    Command::Psync(_) => {
                        let mut receiver = self.sender.subscribe();

                        while let Ok(frame) = receiver.recv().await {
                            self.connection.write_frame(&frame).await?;
                        };
                    }
                    _ => (),
                }
            };
        }
    }

    fn increase_offset(&mut self, increase: usize) {
        let mut offset = self.server_info.replinfo.repl_offset.lock().unwrap();
        *offset += increase as i64;
        drop(offset);
    }

    async fn run_as_cmd(&mut self, frame: &Frame) -> anyhow::Result<Command> {
        let cmd = Command::from_frame(frame)?;
        cmd.apply(
            &mut self.connection,
            &mut self.db,
            &self.server_info,
        ).await?;

        Ok(cmd)
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        if self.connection.is_repl_conn && self.server_info.is_master() {
            let mut count = self.server_info.replinfo.repl_count.lock().unwrap();
            *count -= 1;
        }
    }
}