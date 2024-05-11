use std::error::Error;

use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}};
use tokio::io::AsyncReadExt;


pub struct Redis {
    listener: TcpListener,
}


impl Redis {
    pub async fn new(addr: &str) -> Result<Redis, Box<dyn Error>> {
        Ok(
            Redis {
                listener: TcpListener::bind(&addr).await?,
            }
        )
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> { 
        loop {
            let (socket, _) = self.listener.accept().await?;

            tokio::spawn(async move {
                Redis::handle_connection(socket).await;
            });
        }
    }

    async fn handle_connection(mut socket: TcpStream) {
        let mut command = [0; 1024];

        loop {
            match socket.read(&mut command).await {
                Ok(0) => return,
                Ok(_) => {
                    if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                        eprintln!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                },
                Err(e) =>  {
                    eprintln!("failed to read from socket; err = {:?}", e);
                    return;
                }
            };
        }
    }
}
//  echo -e "PING\nPING" | ./spawn_redis_server.sh
