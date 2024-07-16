use std::io::Cursor;

use bytes::Bytes;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

use super::cmd::ClientCmd;
use super::cmd::get::Get;
use super::cmd::Wait;
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
    replica1_cfg: Config,
    replica2_cfg: Config,

    _mt: JoinHandle<()>,
    _r1t: JoinHandle<()>,
    _r2t: JoinHandle<()>,
}

impl TestSetup {
    async fn setup() -> TestSetup {
        let master_cfg = TestSetup::config("127.0.0.1", "6379", None);
        let replica1_cfg = TestSetup::config("127.0.0.1", "6380", Some(&master_cfg.addr));
        let replica2_cfg = TestSetup::config("127.0.0.1", "6381", Some(&master_cfg.addr));

        let mut master = TestSetup::setup_server(&master_cfg).await;
        let mut replica1 = TestSetup::setup_server(&replica1_cfg).await;
        let mut replica2 = TestSetup::setup_server(&replica2_cfg).await;

        let _mt = tokio::spawn(async move { master.run().await.unwrap() });
        let _r1t = tokio::spawn(async move { replica1.run().await.unwrap() });
        let _r2t = tokio::spawn(async move { replica2.run().await.unwrap() });

        TestSetup {
            master_cfg,
            replica1_cfg,
            replica2_cfg,
            _mt,
            _r1t,
            _r2t,
        }
    }

    fn config(host: &str, port: &str, master: Option<&Addr>) -> Config {
        Config {
            addr: Addr {
                host: host.to_string(),
                port: port.to_string(),
            },
            master_addr: match master {
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

    let input = b"*3\r\n$3\r\nSET\r\n$5\r\ngrape\r\n$9\r\nraspberry\r\n";
    let frame = make_frame(input);

    master_conn.write_frame(&frame).await.unwrap();
    master_conn.read_frame().await.unwrap();

    let wait = Wait {
        numreplicas: 2,
        timeout: 500
    };

    master_conn.write_frame(&wait.to_frame()).await.unwrap();
    master_conn.read_frame().await.unwrap();

    let get = Get::new("grape".to_string());

    let repl1_socket = TcpStream::connect(setup.replica1_cfg.addr.to_string()).await.unwrap();
    let mut repl1_conn = Connection::new(repl1_socket);

    repl1_conn.write_frame(&get.to_frame()).await.unwrap();

    let response_from_repl1 = repl1_conn.read_frame().await.unwrap().unwrap();

    let repl2_socket = TcpStream::connect(setup.replica2_cfg.addr.to_string()).await.unwrap();
    let mut repl2_conn = Connection::new(repl2_socket);

    repl2_conn.write_frame(&get.to_frame()).await.unwrap();

    let response_from_repl2 = repl2_conn.read_frame().await.unwrap().unwrap();

    let expected = Frame::Bulk(Bytes::from_static(b"raspberry"));

    assert_eq!(
        response_from_repl1,
        expected,
    );
    assert_eq!(
        response_from_repl2,
        expected,
    );

    let input = b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
    let frame = make_frame(input);

    master_conn.write_frame(&frame).await.unwrap();
    master_conn.read_frame().await.unwrap();

    let wait = Wait {
        numreplicas: 2,
        timeout: 500
    };

    master_conn.write_frame(&wait.to_frame()).await.unwrap();
    let wait_resp = master_conn.read_frame().await.unwrap().unwrap();

    let expected = Frame::Integer(2);
    assert_eq!(
        wait_resp,
        expected
    );

    let get = Get::new("foo".to_string());

    repl1_conn.write_frame(&get.to_frame()).await.unwrap();
    let response_from_repl1 = repl1_conn.read_frame().await.unwrap().unwrap();

    repl2_conn.write_frame(&get.to_frame()).await.unwrap();
    let response_from_repl2 = repl2_conn.read_frame().await.unwrap().unwrap();

    let expected = Frame::Bulk(Bytes::from_static(b"bar"));

    assert_eq!(
        response_from_repl1,
        expected,
    );
    assert_eq!(
        response_from_repl2,
        expected,
    );
}

#[tokio::test]
async fn test_wait_no_commands() {
    let setup = TestSetup::setup().await;

    sleep(Duration::from_millis(100)).await;

    let master_socket = TcpStream::connect(setup.master_cfg.addr.to_string()).await.unwrap();
    let mut master_conn = Connection::new(master_socket);

    let wait = Wait {
        numreplicas: 3,
        timeout: 500
    };

    master_conn.write_frame(&wait.to_frame()).await.unwrap();
    let wait_resp = master_conn.read_frame().await.unwrap().unwrap();

    let expected = Frame::Integer(2);
    assert_eq!(
        wait_resp,
        expected
    );
}
