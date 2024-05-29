use bytes::Bytes;

use super::*;
use crate::redis::tests::make_frame;


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
        ping.to_response(),
        expected,
    )
}

#[test]
fn test_cmd_ping_with_msg_to_response() {
    let ping = Ping::new(Some(String::from("Hello there")));

    let expected = Frame::Bulk(Bytes::from_static(b"Hello there"));

    assert_eq!(
        ping.to_response(),
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
        echo.to_response(),
        expected,
    )
}
