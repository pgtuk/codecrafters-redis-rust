use std::{env, process};

mod redis;
use redis::{Config, Server};


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::parse(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let mut redis = Server::new(&config).await
        .expect("Failed to connect");
    
    if let Err(e) = redis.run().await {
        eprintln!("Runtime error = {:?}", e);
    }
}
