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
            self.check_wait_lock().await;

            let opt_frame = self.connection.read_frame().await?;
            let frame = match opt_frame {
                Some(frame) => { frame }
                // None means that the socket was closed by peer
                None => return Ok(()),
            };

            let cmd = self.run_as_cmd(&frame).await?;

            self.increase_offset(frame.byte_len()).await;

            if self.server_info.is_master() {
                match cmd {
                    // replicate write commands
                    Command::Set(_) => { self.sender.send(frame)?; }

                    // after psync cmd master starts listening for write commands to replicate
                    Command::Psync(_) => {
                        let mut receiver = self.sender.subscribe();

                        while let Ok(frame) = receiver.recv().await {
                            self.connection.write_frame(&frame).await?;

                            if receiver.is_empty() {
                                self.ack_sync().await;
                            }
                        };
                    }
                    _ => (),
                }
            };
        }
    }

    async fn ack_sync(&self) {
        let mut ack = self.server_info.replinfo.repl_completed.write().await;
        *ack += 1;
    }

    async fn check_wait_lock(&self) -> bool {
        *self.server_info.replinfo.wait_lock.lock().await
    }

    async fn increase_offset(&mut self, increase: usize) {
        let mut offset = self.server_info.replinfo.offset.lock().await;
        *offset += increase as i64;
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
            let mut count = self.server_info.replinfo.count.blocking_lock();
            *count -= 1;
        }
    }
}