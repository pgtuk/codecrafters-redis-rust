// use std::sync::Arc;

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
}

impl Server {
    pub async fn new(cfg: Config) -> Result<Server> {
        Ok(
            Server {
                listener: TcpListener::bind(&cfg.addr()).await?,
                db: Db::new(),
            }
        )
    }

    pub async fn run(&mut self) -> Result<()> { 
        loop {
            let socket = self.accept().await?;
            
            let mut handler = Handler {
                connection: Connection::new(socket),
                db: self.db.clone(),
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

struct Handler {
    connection: Connection,
    db: Db,
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

            cmd.apply(&mut self.connection, &mut self.db).await?;
        }
    }
}

#[cfg(test)]
mod tests;
