use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Duration};

mod connection;
use connection::Connection;

mod cmd;
mod frame;
mod parser;


pub struct Redis {
    listener: TcpListener,
}


impl Redis {
    pub async fn new(addr: &str) -> Result<Redis> {
        Ok(
            Redis {
                listener: TcpListener::bind(&addr).await?,
            }
        )
    }

    pub async fn run(&mut self) -> Result<()> { 
        loop {
            let socket = self.accept().await?;
            
            let mut connection = Connection::new(socket);

            tokio::spawn(async move {
                if let Err(e) = Redis::handle_connection(&mut connection).await {
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

    async fn handle_connection(connection: &mut Connection) -> Result<()> {
        // TODO Write ERROR
        loop {
            let opt_frame =  connection.read_frame().await?;

            let frame = match opt_frame {
                Some(frame) => {frame},
                None => return Ok(()),
            };

            let cmd = cmd::Command::from_frame(frame)?;

            cmd.apply(connection).await?;
        }
    }
}
//  echo -e "PING\nPING" | ./spawn_redis_server.sh
