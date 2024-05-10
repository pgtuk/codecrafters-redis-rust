use std::{
    io::prelude::*, 
    net::TcpListener,
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
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut command = vec![];
                    
                    loop {
                        let read = stream.read(&mut command);
                        match read {
                            Ok(0) => break,
                            Ok(_) => {
                                let response = "+PONG\r\n";

                                stream.write_all(response.as_bytes()).unwrap();
                                stream.flush().unwrap();
                            },
                            Err(_) => panic!("Error while reading command"),
                        };
                    }
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }
}