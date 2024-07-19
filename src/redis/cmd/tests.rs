use std::net::SocketAddr;

use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};

use crate::redis::{
    Config,
    db::Db,
    Role,
    tests::make_frame,
};
use crate::redis::cmd::ClientCmd;
use crate::Server;

use super::*;

fn config() -> Config {
    Config::default()
}

pub(super) async fn start_server() -> SocketAddr {
    // redis server fixture

    let cfg = config();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut server = Server {
        listener,
        db: Db::new(),
        info: ServerInfo::new(cfg, Role::Master),
    };
    tokio::spawn(async move { server.run().await });

    addr
}

pub(super) async fn prepare_conn(addr: SocketAddr) -> Connection {
    let stream = TcpStream::connect(addr).await.unwrap();
    Connection::new(stream)
}

// ECHO
#[test]
fn test_cmd_from_frame_echo() {
    let input = b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(&frame).unwrap();

    let expected = Command::Echo(
        Echo::new(Bytes::from_static(b"hey"))
    );

    assert_eq!(
        cmd,
        expected,
    )
}

#[tokio::test]
async fn test_cmd_echo_to_response() {
    let addr = start_server().await;
    let mut conn = prepare_conn(addr).await;

    let echo = Echo::new(Bytes::from_static(b"hey"));
    conn.write_frame(&echo.to_frame()).await.unwrap();

    let response_frame = conn.read_frame().await.unwrap().unwrap();
    let expected = Frame::Bulk(Bytes::from_static(b"hey"));

    assert_eq!(
        response_frame,
        expected,
    )
}

// SET/GET
#[test]
fn test_cmd_from_frame_set() {
    let input = b"*3\r\n$3\r\nSET\r\n$3\r\nhey\r\n$3\r\nyou\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(&frame).unwrap();

    let expected = Command::Set(
        Set::new(
            "hey".to_string(),
            Bytes::from_static(b"you"),
            None,
        )
    );

    assert_eq!(
        cmd,
        expected,
    )
}

#[test]
fn test_cmd_from_frame_get() {
    let input = b"*2\r\n$3\r\nGET\r\n$3\r\nhey\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(&frame).unwrap();

    let expected = Command::Get(
        Get::new("hey".to_string())
    );

    assert_eq!(
        cmd,
        expected,
    )
}

#[tokio::test]
async fn test_cmd_set_get() {
    let addr = start_server().await;
    let mut conn = prepare_conn(addr).await;

    let set = Set::new(
        "hey".to_string(),
        Bytes::from_static(b"you"),
        None,
    );

    conn.write_frame(&set.to_frame()).await.unwrap();
    let response_frame = conn.read_frame().await.unwrap().unwrap();
    let expected = Frame::Simple("OK".to_string());

    assert_eq!(
        response_frame,
        expected,
    );

    let get = Get::new("hey".to_string());
    conn.write_frame(&get.to_frame()).await.unwrap();

    let response_frame = conn.read_frame().await.unwrap().unwrap();
    let expected = Frame::Bulk(Bytes::from_static(b"you"));

    assert_eq!(
        response_frame,
        expected
    )
}


#[tokio::test]
async fn test_cmd_set_with_ttl() {
    let addr = start_server().await;
    let mut conn = prepare_conn(addr).await;

    let input = b"*5\r\n$3\r\nSET\r\n$5\r\ngrape\r\n$9\r\nraspberry\r\n$2\r\npx\r\n$3\r\n100\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(&frame).unwrap();
    if let Command::Set(set) = cmd {
        conn.write_frame(&set.to_frame()).await.unwrap();
    };

    conn.read_frame().await.unwrap().unwrap();

    let get = Get::new("grape".to_string());
    conn.write_frame(&get.to_frame()).await.unwrap();

    let before_expire = conn.read_frame().await.unwrap().unwrap();
    let expected = Frame::Bulk(Bytes::from_static(b"raspberry"));

    assert_eq!(before_expire, expected);

    sleep(Duration::from_millis(100)).await;

    conn.write_frame(&get.to_frame()).await.unwrap();

    let after_expire = conn.read_frame().await.unwrap().unwrap();
    let expected = Frame::Null;

    assert_eq!(after_expire, expected);
}


#[tokio::test]
async fn test_cmd_info() {
    let addr = start_server().await;
    let mut conn = prepare_conn(addr).await;

    let info = Info::new();
    conn.write_frame(&info.to_frame()).await.unwrap();

    let response = conn.read_frame().await.unwrap().unwrap();

    let expected = Frame::Bulk(Bytes::from_static(
        b"role:master\nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb\nmaster_repl_offset:0"
    ));

    assert_eq!(response, expected)
}
