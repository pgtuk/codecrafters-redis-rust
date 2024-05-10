mod redis;

use redis::Redis;

fn main() {

    let redis = Redis::new("127.0.0.1:6379");
    
    redis.run();
}
