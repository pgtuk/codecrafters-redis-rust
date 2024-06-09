use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use super::utils::Addr;
use super::cmd::Ping;


pub async fn handshake(master_addr: &Addr) -> Result<()> {
    let mut master = TcpStream::connect(master_addr.to_string()).await?;

    let ping = Ping::new(None).to_frame();

    master.write_all(&ping.to_response()).await?;

    // TODO read response from master
    unimplemented!()
}