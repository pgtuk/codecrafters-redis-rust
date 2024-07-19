use anyhow::Result;

use crate::redis::{
    frame::Frame,
    parser::{
        Parser,
        ParserError,
    },
    utils::Named,
};

use super::ClientCmd;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Ping {
    msg: Option<String>,
}

impl Named for Ping {
    const NAME: &'static str = "PING";
}

impl Ping {
    pub fn new(msg: Option<String>) -> Ping {
        Ping { msg }
    }

    pub fn parse_args(parser: &mut Parser) -> Result<Ping> {
        match parser.next_string() {
            Ok(args) => Ok(Ping::new(Some(args))),
            Err(ParserError::EndOfStream) => Ok(Ping::new(None)),
            Err(e) => Err(e.into()),
        }
    }

    pub fn apply(&self) -> Frame {
        match &self.msg {
            None => Frame::Simple("PONG".to_string()),
            Some(msg) => Frame::Bulk(msg.clone().into()),
        }
    }
}

impl ClientCmd for Ping {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        frame.add(Frame::Bulk(Ping::NAME.into()));
        if let Some(msg) = &self.msg {
            frame.add(Frame::Bulk(msg.clone().into()))
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::redis::cmd::Command;
    use crate::redis::cmd::tests::{prepare_conn, start_server};
    use crate::redis::tests::make_frame;

    use super::*;

    #[test]
    fn test_cmd_from_frame_ping_no_msg() {
        let frame = make_frame(b"*1\r\n$4\r\nPING\r\n");

        let cmd = Command::from_frame(&frame).unwrap();

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

        let cmd = Command::from_frame(&frame).unwrap();

        let expected = Command::Ping(
            Ping::new(Some(String::from("hello")))
        );

        assert_eq!(
            cmd,
            expected,
        )
    }

    #[tokio::test]
    async fn test_cmd_ping_no_msg() {
        let addr = start_server().await;
        let mut conn = prepare_conn(addr).await;

        let ping = Ping::new(None);
        conn.write_frame(&ping.to_frame()).await.unwrap();

        let response_frame = conn.read_frame().await.unwrap().unwrap();
        let expected = Frame::Simple(String::from("PONG"));
        assert_eq!(
            response_frame,
            expected
        )
    }

    #[tokio::test]
    async fn test_cmd_ping_with_msg_to_response() {
        let addr = start_server().await;
        let mut conn = prepare_conn(addr).await;

        let ping = Ping::new(Some(String::from("Hello there")));
        conn.write_frame(&ping.to_frame()).await.unwrap();

        let response_frame = conn.read_frame().await.unwrap().unwrap();
        let expected = Frame::Bulk(Bytes::from_static(b"Hello there"));

        assert_eq!(
            response_frame,
            expected,
        )
    }
}