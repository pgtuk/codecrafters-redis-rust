use std::io::Cursor;

use bytes::Bytes;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

use super::cmd::ClientCmd;
use super::cmd::get::Get;
use super::config::Config;
use super::Connection;
use super::frame::Frame;
use super::Server;
use super::utils::Addr;

pub fn make_frame(input: &[u8]) -> Frame {
    let mut cursor = Cursor::new(&input[..]);

    Frame::parse(&mut cursor).unwrap()
}


struct TestSetup {
    master_cfg: Config,
    slave_cfg: Config,

    _mt: JoinHandle<()>,
    _st: JoinHandle<()>,
}

impl TestSetup {
    async fn setup() -> TestSetup {
        let master_cfg = TestSetup::config("127.0.0.1", "6379", None);
        let slave_cfg = TestSetup::config("127.0.0.1", "6380", Some(&master_cfg.addr));

        let mut master = TestSetup::setup_server(&master_cfg).await;
        let mut slave = TestSetup::setup_server(&slave_cfg).await;

        let _mt = tokio::spawn(async move { master.run().await.unwrap() });
        let _st = tokio::spawn(async move { slave.run().await.unwrap() });

        TestSetup {
            master_cfg,
            slave_cfg,
            _mt,
            _st,
        }
    }

    fn config(host: &str, port: &str, repl_of: Option<&Addr>) -> Config {
        Config {
            addr: Addr {
                host: host.to_string(),
                port: port.to_string(),
            },
            repl_of: match repl_of {
                Some(addr) => Some(addr.clone()),
                None => None
            },
        }
    }

    async fn setup_server(cfg: &Config) -> Server {
        Server::setup(cfg).await.unwrap()
    }
}


#[tokio::test]
async fn test_replication() {
    let setup = TestSetup::setup().await;

    sleep(Duration::from_millis(100)).await;

    let master_socket = TcpStream::connect(setup.master_cfg.addr.to_string()).await.unwrap();
    let mut master_conn = Connection::new(master_socket);
    let get = Get::new("grape".to_string());
    let input = b"*3\r\n$3\r\nSET\r\n$5\r\ngrape\r\n$9\r\nraspberry\r\n";
    let frame = make_frame(input);

    master_conn.write_frame(&frame).await.unwrap();

    master_conn.read_frame().await.unwrap();

    master_conn.write_frame(&get.to_frame()).await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let slave_socket = TcpStream::connect(setup.slave_cfg.addr.to_string()).await.unwrap();
    let mut slave_conn = Connection::new(slave_socket);

    slave_conn.write_frame(&get.to_frame()).await.unwrap();

    let response = slave_conn.read_frame().await.unwrap().unwrap();
    let expected = Frame::Bulk(Bytes::from_static(b"raspberry"));

    assert_eq!(
        response,
        expected,
    )
}
