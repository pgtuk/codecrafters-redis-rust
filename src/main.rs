mod redis;

use redis::Redis;

#[tokio::main]
async fn main() {

    let mut redis = Redis::new("127.0.0.1:6379").await
        .expect("Failed to connect");
    
    if let Err(e) = redis.run().await {
        eprintln!("Runtime error = {:?}", e);
    }
}
