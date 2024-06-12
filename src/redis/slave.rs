use anyhow::{Result, bail};
use tokio::net::TcpStream;

use super::utils::Addr;
use super::cmd::Ping;
use super::connection::Connection;


pub async fn handshake(master_addr: &Addr) -> Result<(), > {
    let socket = TcpStream::connect(master_addr.to_string()).await?;

    let mut conn = Connection::new(socket);

    let ping = Ping::new(None).to_frame();
    
    conn.write_frame(&ping).await?;

    match conn.read_frame().await? {
        Some(_) => Ok(()),
        None => bail!("No PONG from master")
    }
}