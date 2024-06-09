use bytes::Bytes;

use tokio::time::{sleep, Duration};

use super::*;
use crate::redis::{
    db::Db, 
    tests::make_frame,
    Role
};
    
    
    // PING
#[test]
fn test_cmd_from_frame_ping_no_msg() {
    let frame = make_frame(b"*1\r\n$4\r\nPING\r\n");

    let cmd = Command::from_frame(frame).unwrap();

    let expected = Command::Ping(
        Ping::new(None)
    );

    assert_eq!(
        cmd,
        expected,
    )
}

#[test]
fn test_cmd_from_frame_ping_with_msg() {
    let frame = make_frame(b"*2\r\n$4\r\nPING\r\n$5\r\nhello\r\n");

    let cmd = Command::from_frame(frame).unwrap();

    let expected = Command::Ping(
        Ping::new(Some(String::from("hello")))
    );

    assert_eq!(
        cmd,
        expected,
    )
}

#[test]
fn test_cmd_ping_no_msg_to_response() {
    let ping = Ping::new(None);

    let expected = Frame::Simple(String::from("PONG"));

    assert_eq!(
        ping.apply(),
        expected,
    )
}

#[test]
fn test_cmd_ping_with_msg_to_response() {
    let ping = Ping::new(Some(String::from("Hello there")));

    let expected = Frame::Bulk(Bytes::from_static(b"Hello there"));

    assert_eq!(
        ping.apply(),
        expected,
    )
}

// ECHO
#[test]
fn test_cmd_from_frame_echo() {
    let input = b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(frame).unwrap();

    let expected = Command::Echo(
        Echo::new(Bytes::from_static(b"hey"))
    );

    assert_eq!(
        cmd,
        expected,
    )
}

#[test]
fn test_cmd_echo_to_response() {
    let echo = Echo::new(Bytes::from_static(b"hey"));

    let expected = Frame::Bulk(Bytes::from_static(b"hey"));

    assert_eq!(
        echo.apply(),
        expected,
    )
}

// SET/GET
#[test]
fn test_cmd_from_frame_set() {
    let input = b"*3\r\n$3\r\nSET\r\n$3\r\nhey\r\n$3\r\nyou\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(frame).unwrap();

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

    let cmd = Command::from_frame(frame).unwrap();

    let expected = Command::Get(
        Get::new("hey".to_string())
    );

    assert_eq!(
        cmd,
        expected,
    )
}

#[tokio::test]
async fn test_cmd_set() {
    let mut db = Db::new();

    let set = Set::new(
        "hey".to_string(), 
        Bytes::from_static(b"you"), 
        None,
    );

    set.apply(&mut db);

    let expected = Bytes::from_static(b"you");    

    assert_eq!(
        db.get("hey").unwrap(),
        expected,
    )
}

#[tokio::test]
async fn test_cmd_get() {
    let mut db = Db::new();

    let set = Set::new(
        "hey".to_string(),
        Bytes::from_static(b"you"), 
        None,
    );
    set.apply(&mut db);

    let get = Get::new("hey".to_string());
    let data = get.apply(&mut db);
    
    let expected = Frame::Bulk(Bytes::from_static(b"you"));    

    assert_eq!(
        data,
        expected,
    )
}

#[tokio::test]
async fn test_cmd_set_ttl() {
    let mut db = Db::new();

    let input = b"*5\r\n$3\r\nSET\r\n$5\r\ngrape\r\n$9\r\nraspberry\r\n$2\r\npx\r\n$3\r\n100\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(frame).unwrap();
    if let Command::Set(cmd) = cmd {
        cmd.apply(&mut db);
    };

    let get = Get::new("grape".to_string());
    let data = get.apply(&mut db);

    let expected = Frame::Bulk(Bytes::from_static(b"raspberry"));  
    assert_eq!(data, expected);
}

#[tokio::test]
async fn test_cmd_set_ttl_expire() {
    let mut db = Db::new();

    let input = b"*5\r\n$3\r\nSET\r\n$5\r\ngrape\r\n$9\r\nraspberry\r\n$2\r\npx\r\n$3\r\n100\r\n";
    let frame = make_frame(input);

    let cmd = Command::from_frame(frame).unwrap();
    if let Command::Set(cmd) = cmd {
        cmd.apply(&mut db);
    };

    sleep(Duration::from_millis(110)).await;

    let get = Get::new("grape".to_string());
    let data = get.apply(&mut db);

    let expected = Frame::Null;  
    assert_eq!(data, expected);
}

#[test]
fn test_cmd_info () {
    let info = ServerInfo::new(
        Role::Slave,
        None,
    );
    let cmd = Info::new();

    let frame = cmd.apply(&info);

    let expected = Frame::Bulk(Bytes::from_static(
        b"$10\r\nrole:slave\r\n"
    ));
    
    assert_eq!(frame, expected)
}
