use std::error::Error;

mod redis;

use redis::Redis;

pub type ProtocolError = Box<dyn Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, ProtocolError>;


#[tokio::main]
async fn main() {

    let mut redis = Redis::new("127.0.0.1:6379").await
        .expect("Failed to connect");
    
    if let Err(e) = redis.run().await {
        eprintln!("Runtime error = {:?}", e);
    }
    
}
