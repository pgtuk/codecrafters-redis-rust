use super::*;
use crate::redis::tests::make_frame;

// test parse
#[test]
fn test_parse_simple () {
    let frame = make_frame(b"+OK\r\n");

    let expected = Frame::Simple(
        String::from("OK")
    );

    assert_eq!(expected, frame);
}

#[test]
fn test_parse_bulk () {
    let frame = make_frame(b"$5\r\nhello\r\n");

    let expected = Frame::Bulk(
        Bytes::from_static(b"hello")
    );

    assert_eq!(expected, frame);
}

#[test]
fn test_parse_empty_bulk () {
    let frame = make_frame(b"$0\r\n\r\n");

    let expected = Frame::Bulk(
        Bytes::from_static(b"")
    );
    
    assert_eq!(expected, frame);
}

#[test]
fn test_parse_null_bulk () {
    let frame = make_frame(b"$-1\r\n");

    let expected = Frame::Null;
    
    assert_eq!(expected, frame);
}

#[test]
fn test_parse_array () {
    let frame = make_frame(b"*2\r\n$4\r\nPING\r\n$-1\r\n");
    
    let expected = Frame::Array(vec![
        Frame::Bulk(Bytes::from_static(b"PING")),
        Frame::Null
    ]);
    
    assert_eq!(expected, frame);
}


// test to_response
#[test]
fn test_to_response_simple() {
    let input = b"+OK\r\n";
    let frame = make_frame(input);
    let response = frame.to_response();

    assert_eq!(response, input)
}

#[test]
fn test_to_response_null() {
    let input = b"$-1\r\n";
    let frame = make_frame(input);
    let response = frame.to_response();

    assert_eq!(response, input)
}

#[test]
fn test_to_response_bulk() {
    let input = b"$5\r\nhello\r\n";
    let frame = make_frame(input);
    let response = frame.to_response();

    assert_eq!(response, input)
}