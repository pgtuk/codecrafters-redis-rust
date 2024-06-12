use std::io::Cursor;

use tokio::time::{self, Duration};

use super::config::Config;
use super::frame::Frame;
use super::Server;
use super::utils::Addr;

pub fn make_frame(input: &[u8]) -> Frame {
    let mut cursor = Cursor::new(&input[..]);

    Frame::parse(&mut cursor).unwrap()
}

#[tokio::test]
async fn test_master_slave_handshake() {
    let host = String::from("127.0.0.1");
    let master_port = String::from("6379");
    let slave_port = String::from("6380");

    let master_cfg = Config {
        addr: Addr { host: host.clone(), port: master_port.clone() },
        replicaof: None
    };
    let slave_cfg = Config {
        addr: Addr { host: host.clone(), port: slave_port },
        replicaof: Some(Addr { host: host.clone(), port: master_port })
    };

    let mut master = Server::setup(&master_cfg).await.unwrap();
    let mut slave = Server::setup(&slave_cfg).await.unwrap();

    let mt = tokio::spawn(async move { master.run().await.unwrap() });
    let st = tokio::spawn(async move { slave.run().await.unwrap() });

    time::sleep(Duration::from_millis(100)).await;

    st.abort();
    mt.abort();
}