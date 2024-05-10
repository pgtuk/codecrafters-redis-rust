use std::{
    io::prelude::*, 
    net::{TcpListener, TcpStream},
};


pub struct Redis {
    listener: TcpListener,
}


impl Redis {
    pub fn new(addr: &str) -> Redis {
        let listener = TcpListener::bind(addr).unwrap();

        Redis { listener }
    }

    pub fn run(&self) {
        println!("RUN");
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    Redis::handle_connection(stream);
                }
                Err(e) => {
                    println!("Error while processing stream: {}", e);
                }
            }
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut command = [0; 512];

        loop {
            let read = stream.read(&mut command).unwrap();

            if read == 0 {
                break;
            } else {
                stream.write(b"+PONG\r\n").unwrap();
            }

        }
    }
}

//  echo -e "PING\nPING" | ./spawn_redis_server.sh
