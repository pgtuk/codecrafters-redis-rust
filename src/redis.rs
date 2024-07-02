use core::fmt;
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Duration};

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


pub struct Server {
    listener: TcpListener,
    db: Db,
    info: ServerInfo,
}

impl Server {
    fn new(listener: TcpListener, db: Db, info: ServerInfo) -> Server {
        Server {
            listener,
            db,
            info,
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
            Role::Slave => self.connect_to_master().await?,
            Role::Master => ()
        };

        Ok(())
    }

    fn as_role(&self) {

    }

    pub async fn run(&mut self) -> Result<()> {
        self.on_startup().await?;

        loop {
            let socket = self.accept().await?;
            self.handle_connection(Connection::new(socket)).await;
        }
    }

    async fn handle_connection(&self, conn: Connection) {
        let mut handler = Handler::new(
            conn,
            self.db.clone(),
            self.info.clone(),
        );

        tokio::spawn(async move {
            if let Err(e) = handler.run().await {
                eprintln!("Error while handling connection: {}", e);
            };
        });
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

    async fn connect_to_master(&self) -> Result<()>{
        match &self.info.replinfo.replicaof {
            None => bail!("No master address"),
            Some(master_addr) => {
                let master_conn = slave::handshake(&self.info, master_addr).await?;

                self.handle_connection(master_conn).await;
            },
        }

        Ok(())
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
