use core::fmt;
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Duration};

mod connection;
use connection::Connection;
mod cmd;
mod config;
pub use config::Config;
mod db;
use db::Db;
mod frame;
mod parser;
mod slave;
mod utils;



pub struct Server {
    listener: TcpListener,
    db: Db,
    info: ServerInfo,
}

impl Server {
    fn new(listener: TcpListener, db: Db, info: ServerInfo) -> Server {
        Server { listener, db, info }
    }

    pub async fn setup(cfg: &Config) -> Result<Server> {
        let role = match &cfg.replicaof {
            Some(_) => Role::Slave,
            None => Role::Master
        };

        let server = Server::new(
            TcpListener::bind(cfg.addr.to_string()).await?,
            Db::new(),
            ServerInfo::new(cfg.addr.clone(), role, cfg.replicaof.clone())
        );

        Ok(server)
    }

    async fn on_startup(&self) -> Result<()> {
        match self.info.role {
            Role::Master => Ok(()),
            Role::Slave => {
                match &self.info.replinfo.replicaof {
                    Some(master_addr) => Ok(slave::handshake(&self.info, master_addr).await?),
                    None => bail!("No master address"),
                }
            },
        }
    }

    pub async fn run(&mut self) -> Result<()> { 
        self.on_startup().await?;

        loop {
            let socket = self.accept().await?;
            
            let mut handler = Handler {
                connection: Connection::new(socket),
                db: self.db.clone(),
                info: self.info.clone(),
            };

            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection().await {
                    eprintln!("Error while handling connection: {}", e);
                };
            });
        }
    }

    async fn accept(&mut self) -> Result<TcpStream> {
        let mut tries = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if tries > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(tries)).await;

            tries *= 2;
        }
    }
}

#[derive(Clone)]
pub enum Role {
    Master,
    Slave,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Role::Master => write!(f, "master"),
            Role::Slave => write!(f, "slave"),
        }
    }
}

#[derive(Clone)]
pub struct ServerInfo {
    addr: utils::Addr,
    role: Role,
    replinfo: Arc<Replinfo>,
}

impl ServerInfo {
    fn new(addr: utils::Addr, role: Role, replicaof: Option<utils::Addr>) -> ServerInfo {
        ServerInfo {
            addr,
            role, 
            replinfo: Arc::new(Replinfo {
                repl_id: String::from("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"),
                repl_offset: Mutex::new(0),
                replicaof,
            }),
            
        }
    }
}

pub struct Replinfo {
    repl_id: String,
    repl_offset: Mutex<i64>,
    replicaof: Option<utils::Addr>,
}

struct Handler {
    connection: Connection,
    db: Db,
    info: ServerInfo,
}

impl Handler {
    async fn handle_connection(&mut self) -> Result<()> {
        // TODO Write ERROR
        loop {
            let opt_frame =  self.connection.read_frame().await?;
            
            let frame = match opt_frame {
                Some(frame) => {frame},
                None => return Ok(()),
            };

            let cmd = cmd::Command::from_frame(frame)?;

            cmd.apply(&mut self.connection, &mut self.db, &self.info).await?;
        }
    }
}


#[cfg(test)]
mod tests;
