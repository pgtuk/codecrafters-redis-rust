use anyhow::{Result, bail};
use tokio::net::TcpStream;

use super::utils::Addr;
use super::cmd::{
    client_cmd::ClientCmd,
    replconf::{Replconf, ReplconfParam}, 
    Ping,
};
use super::connection::Connection;


pub async fn handshake(master_addr: &Addr) -> Result<(), > {
    let socket = TcpStream::connect(master_addr.to_string()).await?;
    let mut conn = Connection::new(socket);
    
    sequence_step(
        &Ping::new(None),
        &mut conn, 
    ).await?;

    sequence_step(
        &Replconf { param: ReplconfParam::ListeningPort, arg: master_addr.port.clone() },
        &mut conn, 
    ).await?;

    sequence_step(
        &Replconf { param: ReplconfParam::Capa, arg: "psync2".to_string() },
        &mut conn, 
    ).await?;

    Ok(())
}

async fn sequence_step(cmd: &impl ClientCmd, conn: &mut Connection) -> Result<()>{
    let cmd_as_frame = cmd.to_frame();

    conn.write_frame(&cmd_as_frame).await?;

    match conn.read_frame().await? {
        Some(_) => Ok(()),
        None => bail!("No response from master")
    }
}