use bytes::Bytes;

use crate::redis::frame::Frame;
use super::*;


#[test]
fn test_create_parser() {
    let empty_frame = Frame::Null;
    let simple_frame = Frame::Simple("ok".to_string());
    
    let frame = Frame::Array(vec![
        empty_frame.clone(),
        simple_frame.clone(),
    ]);
    
    let mut parser = Parser::new(&frame).unwrap();
    
    assert_eq!(*parser.next().unwrap(), empty_frame);
    assert_eq!(*parser.next().unwrap(), simple_frame);
    assert_eq!(
        parser.next().unwrap_err(),
        ParserError::EndOfStream
    );
}

#[test]
fn test_create_parser_err() {
    let non_array_frame = Frame::Null;

    let parse_result = Parser::new(&non_array_frame);

    assert_eq!(
        parse_result.unwrap_err(),
        ParserError::Other(
            format!("protocol error; expected array, got {:?}", non_array_frame)
        ),
    )
}

#[test]
fn test_parser_next_string_simple() {
    let string = String::from("Hello there");
    let frame = Frame::Simple(string.clone());

    let frame_array = Frame::Array(vec![
        frame
    ]);
    let mut parser = Parser::new(
        &frame_array
    ).unwrap();

    assert_eq!(
        parser.next_string().unwrap(),
        string,
    )
}


#[test]
fn test_parser_next_string_bulk() {
    let bytes = b"Hello there";
    let frame = Frame::Bulk(
        Bytes::from_static(bytes)
    );
    let frame_array = Frame::Array(vec![frame]);

    let mut parser = Parser::new(&frame_array).unwrap();

    assert_eq!(
        parser.next_string().unwrap(),
        String::from_utf8(bytes.to_vec()).unwrap(),
    )
}
