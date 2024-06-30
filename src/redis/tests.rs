use std::io::Cursor;

use tokio::net::TcpStream;
use tokio::time::{self, Duration};

use super::Connection;
use super::config::Config;
use super::frame::Frame;
use super::Server;
use super::utils::Addr;
use super::cmd::get::Get;
use super::cmd::ClientCmd;


pub fn make_frame(input: &[u8]) -> Frame {
    let mut cursor = Cursor::new(&input[..]);

    Frame::parse(&mut cursor).unwrap()
}

fn config (host: &str, port: &str, replicaof: Option<&Addr>) -> Config {
    Config {
        addr: Addr { 
            host: host.to_string(),
            port: port.to_string(),        
        },
        replicaof: match replicaof {
            Some(addr) => Some(addr.clone()),
            None => None
        }
    }
}

async fn setup_server(cfg: &Config) -> Server {
    Server::setup(cfg).await.unwrap()
}

#[tokio::test]
async fn test_master_slave_handshake() {

    let master_cfg = config("127.0.0.1", "6379", None);
    let slave_cfg = config("127.0.0.1", "6380", Some(&master_cfg.addr));

    let mut master = setup_server(&master_cfg).await;
    let mut slave = setup_server(&slave_cfg).await;

    let mt = tokio::spawn(async move { master.run().await.unwrap() });
    let st = tokio::spawn(async move { slave.run().await.unwrap() });

    time::sleep(Duration::from_millis(100)).await;

    st.abort();
    mt.abort();  
}

#[tokio::test]
async fn test_replication() {
    let master_cfg = config("127.0.0.1", "6379", None);
    let slave_cfg = config("127.0.0.1", "6380", Some(&master_cfg.addr));

    let mut master = setup_server(&master_cfg).await;
    let mut slave = setup_server(&slave_cfg).await;

    let _mt = tokio::spawn(async move { master.run().await.unwrap() });
    let _st = tokio::spawn(async move { slave.run().await.unwrap() });

    let master_socket = TcpStream::connect(master_cfg.addr.to_string()).await.unwrap();

    let mut master_conn = Connection::new(master_socket);

    let input = b"*3\r\n$3\r\nSET\r\n$5\r\ngrape\r\n$9\r\nraspberry\r\n";
    let frame = make_frame(input);

    master_conn.write_frame(&frame).await.unwrap();

    let slave_socket =  TcpStream::connect(slave_cfg.addr.to_string()).await.unwrap();
    let mut slave_conn =  Connection::new(slave_socket);

    let get = Get::new("grape".to_string());
    slave_conn.write_frame(&get.to_frame()).await.unwrap();

    match slave_conn.read_frame().await {
        Ok(result) => {
            dbg!(result);
            Ok(())
        }
        Err(e) => {
            dbg!(e);
            Ok(())
        }
    }

    // dbg!(get_result);
}