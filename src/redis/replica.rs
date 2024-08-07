use std::sync::Arc;

use anyhow::{bail, Result};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

use crate::redis::frame::Frame;

use super::{
    cmd::{
        ClientCmd,
        Ping,
        Psync,
        replconf::{Replconf, ReplconfParam},
    },
    connection::Connection,
    ServerInfo,
    utils::{Addr, Named},
};

#[derive(Debug, Clone)]
pub(crate) enum ReplicationMsg {
    Propagate(Frame),
    Wait(u64),
}

#[derive(Clone)]
pub(crate) struct Replinfo {
    pub id: String,
    pub offset: Arc<Mutex<i64>>,
    // number of connected replicas
    pub count: Arc<RwLock<i8>>,
    pub master: Option<Addr>,

    pub wait_lock: Arc<Mutex<bool>>,
    pub repl_completed: Arc<RwLock<i8>>,
    pub pending_commands: Arc<RwLock<bool>>,
}

impl Replinfo {
    pub(crate) async fn add_replica(&mut self) {
        let mut count = self.count.write().await;
        *count += 1
    }

    pub(crate) async fn drop_replica(&mut self) {
        let mut count = self.count.write().await;
        *count -= 1
    }
    
    pub(crate) async fn has_pending(&self) -> bool {
        let pending = self.pending_commands.read().await;

        *pending
    }

    // pub(crate) fn blocking_drop_replica(&mut self) {
    //     let mut count = self.count.blocking_write();
    //     *count -= 1
    // }
}

pub async fn handshake(slave_info: &ServerInfo, master_addr: &Addr) -> Result<Connection> {
    let socket = TcpStream::connect(master_addr.to_string()).await?;
    let mut conn = Connection::new(socket);
    conn.is_repl_conn = true;

    sequence_step(
        &Ping::new(None),
        &mut conn,
    ).await?;

    sequence_step(
        &Replconf { param: ReplconfParam::ListeningPort, arg: slave_info.addr.port.clone() },
        &mut conn,
    ).await?;

    sequence_step(
        &Replconf { param: ReplconfParam::Capa, arg: "psync2".to_string() },
        &mut conn,
    ).await?;

    sequence_step(
        &Psync::default(),
        &mut conn,
    ).await?;

    let _ = conn.read_rdb().await;

    Ok(conn)
}

async fn sequence_step(cmd: &(impl ClientCmd + Named), conn: &mut Connection) -> Result<()> {
    let cmd_as_frame = cmd.to_frame();

    conn.write_frame(&cmd_as_frame).await?;

    match conn.read_frame().await? {
        Some(_) => Ok(()),
        None => bail!(format!("No response from master to {}", cmd.name()))
    }
}