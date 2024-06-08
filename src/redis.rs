use core::fmt;
use std::fmt::Formatter;

use anyhow::Result;

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


pub struct Server {
    listener: TcpListener,
    db: Db,
    info: ServerInfo,
}

impl Server {
    pub async fn new(cfg: &Config) -> Result<Server> {
        let addr = cfg.addr();
        let role = match &cfg.replicaof {
            Some(_) => Role::Slave,
            None => Role::Master
        };
        Ok(
            Server {
                listener: TcpListener::bind(addr).await?,
                db: Db::new(),
                info: ServerInfo { role }
            }
        )
    }

    pub async fn run(&mut self) -> Result<()> { 
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
    role: Role,
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
