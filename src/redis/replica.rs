use std::sync::Mutex;

use anyhow::{bail, Result};
use tokio::net::TcpStream;

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

pub(crate) struct Replinfo {
    pub repl_id: String,
    pub repl_offset: Mutex<i64>,
    pub repl_of: Option<Addr>,
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