use core::fmt;
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use channels::ChannelManager;
pub use config::Config;
use connection::{Connection, Handler};
use db::Db;

mod connection;
mod cmd;
mod config;
mod db;
mod frame;
mod parser;
mod slave;
mod utils;
mod channels;

type ReplConnectionPool = Arc<Mutex<Vec<&'static Connection>>>;

pub struct Server {
    listener: TcpListener,
    db: Db,
    info: ServerInfo,

    repl_connection_pool: Option<ReplConnectionPool>
}

impl Server {
    fn new(listener: TcpListener, db: Db, info: ServerInfo) -> Server {
        Server {
            listener,
            db,
            info,
            repl_connection_pool: None
        }
    }

    pub async fn setup(cfg: &Config) -> Result<Server> {
        let role = match &cfg.replicaof {
            Some(_) => Role::Slave,
            None => Role::Master
        };

        let server = Server::new(
            TcpListener::bind(cfg.addr.to_string()).await?,
            Db::new(),
            ServerInfo::new(cfg.addr.clone(), role, cfg.replicaof.clone()),
        );

        Ok(server)
    }

    async fn on_startup(&mut self) -> Result<()> {
        match self.info.role {
            Role::Slave => {
                match &self.info.replinfo.replicaof {
                    Some(master_addr) => Ok(slave::handshake(&self.info, master_addr).await?),
                    None => bail!("No master address"),
                }
            },
            Role::Master => {
                self.repl_connection_pool = Some(Arc::new(Mutex::new(vec![])));
                Ok(())
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // TODO: use connection pool

        // we set up a connection to master in handshake and must use it     
        self.on_startup().await?;

        // provide a clone of sender to each connection handler to contact ChannelManager
        let (sender, receiver) = mpsc::channel(32);
        // TODO: dedicate some time to naming!
        let mut manager = ChannelManager::new(receiver);
        tokio::spawn(async move {
            manager.run().await
        });

        loop {
            let socket = self.accept().await?;
            
            let mut handler = Handler::new(
                Connection::new(socket),
                self.db.clone(),
                self.info.clone(),
                sender.clone(),
            );

            tokio::spawn(async move {
                if let Err(e) = handler.run().await {
                    eprintln!("Error while handling connection: {}", e);
                };
            });
        }

        // drop(tx)
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

#[derive(Clone, PartialEq)]
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

#[cfg(test)]
mod tests;
