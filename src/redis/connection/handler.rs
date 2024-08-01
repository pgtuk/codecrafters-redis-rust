use std::sync::Arc;

use tokio::sync::broadcast::Sender;
use tokio::time::{Duration, timeout};

use crate::redis::cmd::{ClientCmd, Command};
use crate::redis::cmd::replconf::Replconf;
use crate::redis::connection::Connection;
use crate::redis::db::Db;
use crate::redis::replica::ReplicationMsg;
use crate::redis::ServerInfo;

pub struct Handler {
    pub(crate) connection: Connection,
    db: Db,
    pub(crate) server_info: ServerInfo,
    sender: Arc<Sender<ReplicationMsg>>,
}

impl Handler {
    pub(crate) fn new(
        connection: Connection,
        db: Db,
        server_info: ServerInfo,
        sender: Arc<Sender<ReplicationMsg>>,
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

            let cmd = Command::from_frame(&frame)?;

            cmd.apply(
                // TODO just pass Handler at this point???
                &mut self.connection,
                &mut self.db,
                &mut self.server_info,
                &mut self.sender,
            ).await?;

            // TODO check list of commands which should change offset
            self.increase_offset(frame.byte_len()).await;

            if self.server_info.is_master() {
                match cmd {
                    // replicate write commands
                    Command::Set(_) => {
                        self.sender.send(ReplicationMsg::Propagate(frame))?;
                        self.set_pending(true).await;
                    },

                    // after psync cmd master starts handle_propagationlistening for write commands to replicate
                    Command::Psync(_) => { self.handle_replication().await? }
                    _ => (),
                }
            };
        }
    }

    async fn handle_replication(&mut self) -> anyhow::Result<()> {
        let mut receiver = self.sender.subscribe();
        let getack = Replconf::getack();

        while let Ok(msg) = receiver.recv().await {
            match msg {
                ReplicationMsg::Propagate(frame) => {
                    self.connection.write_frame(&frame).await?;
                },
                ReplicationMsg::Wait(wait_timeout) => {
                    self.connection.write_frame(&getack.to_frame()).await?;

                    match timeout(
                        Duration::from_millis(wait_timeout),
                        self.connection.read_frame(),
                    ).await {
                        Ok(_) => self.ack_sync().await,
                        _ => (),
                    };
                }
            }
        };
        Ok(())
    }


    async fn set_pending(&mut self, val: bool) {
        let mut pending = self.server_info.replinfo.pending_commands.write().await;

        *pending = val
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
}
