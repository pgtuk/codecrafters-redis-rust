use std::sync::Arc;

use anyhow::{bail, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio::sync::broadcast::{self, Sender};
use tokio::time::{self, Duration};

pub use config::Config;
use connection::{Connection, Handler};
use db::Db;
use replica::{ReplicationMsg, Replinfo};
use role::Role;

mod cmd;
mod connection;
mod config;
mod db;
mod frame;
mod parser;
mod replica;
mod role;
mod utils;


pub struct Server {
    listener: TcpListener,
    db: Db,
    info: ServerInfo,
}

impl Server {
    pub async fn setup(cfg: &Config) -> Result<Server> {
        let role = match &cfg.master_addr {
            Some(_) => Role::Slave,
            None => Role::Master
        };

        Ok(
            Server {
                listener: TcpListener::bind(cfg.addr.to_string()).await?,
                db: Db::new(),
                info: ServerInfo::new(cfg.addr.clone(), role, cfg.master_addr.clone()),
            }
        )
    }

    pub async fn run(&mut self) -> Result<()> {
        let (sender_tx, _rx) = broadcast::channel(32);
        let sender = Arc::new(sender_tx);

        let master_conn = self.on_startup().await;
        if let Some(connection) = master_conn {
            // deal replication connection
            self.handle_connection(connection, sender.clone()).await;
        }

        loop {
            let socket = self.accept().await?;
            self.handle_connection(Connection::new(socket), sender.clone()).await;
        }
    }

    async fn on_startup(&mut self) -> Option<Connection> {
        match self.info.role {
            Role::Slave => Some(self.connect_to_master().await.ok()?),
            Role::Master => None
        }
    }

    async fn connect_to_master(&self) -> Result<Connection> {
        match &self.info.replinfo.master {
            None => bail!("No master address"),
            Some(master_addr) => {
                Ok(replica::handshake(&self.info, master_addr).await?)
            }
        }
    }

    async fn handle_connection(
        &self,
        conn: Connection,
        sender: Arc<Sender<ReplicationMsg>>,
    ) {
        let mut handler = Handler::new(
            conn,
            self.db.clone(),
            self.info.clone(),
            sender,
        );

        tokio::spawn(async move {
            if let Err(e) = handler.handle_connection().await {
                eprintln!("Error while handling connection: {}", e);
            };

            if handler.connection.is_repl_conn && handler.server_info.is_master() {
                handler.server_info.replinfo.drop_replica().await;
            }
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
}

#[derive(Clone)]
pub struct ServerInfo {
    addr: utils::Addr,
    role: Role,
    replinfo: Replinfo,
}

impl ServerInfo {
    fn new(addr: utils::Addr, role: Role, master: Option<utils::Addr>) -> ServerInfo {
        ServerInfo {
            addr,
            role,
            replinfo: Replinfo {
                id: String::from("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"),
                offset: Arc::new(Mutex::new(0)),
                count: Arc::new(RwLock::new(0)),
                master,
                wait_lock: Arc::new(Mutex::new(false)),
                repl_completed: Arc::new(RwLock::new(0)),
                pending_commands: false
            },
        }
    }

    pub fn is_master(&self) -> bool {
        self.role == Role::Master
    }
}

#[cfg(test)]
mod tests;