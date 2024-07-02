use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;

use crate::redis::{
    db::Db,
    frame::Frame,
    parser::{
        Parser,
        ParserError,
    },
};
use crate::redis::cmd::ClientCmd;
use crate::redis::utils::{int_as_bytes, Named};

#[derive(Debug, PartialEq, Clone)]
pub struct Set {
    key: String,
    value: Bytes,

    expire: Option<Duration>,
}

impl Named for Set {
    const NAME: &'static str = "SET";
}

impl Set {
    pub fn new(key: String, value: Bytes, expire: Option<Duration>) -> Set {
        Set { key, value, expire }
    }

    pub fn parse_args(parser: &mut Parser) -> Result<Set> {
        let key = parser.next_string()?;
        let value = parser.next_bytes()?;

        let mut expire = None;

        match parser.next_string() {
            Ok(s) if s.to_lowercase() == "px" => {
                let millis = parser.next_int()?;
                expire = Some(Duration::from_millis(millis))
            }
            Ok(_) => {}
            Err(ParserError::EndOfStream) => {}
            Err(e) => return Err(e.into())
        }

        Ok(Set::new(key, value, expire))
    }

    pub fn apply(&self, db: &mut Db) -> Frame {
        db.set(
            self.key.clone(),
            self.value.clone(),
            self.expire,
        );

        Frame::Simple("OK".to_string())
    }
}

impl ClientCmd for Set {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        frame.add(Frame::Bulk(Set::NAME.into()));
        frame.add(Frame::Bulk(self.key.clone().into()));
        frame.add(Frame::Bulk(self.value.clone()));

        if let Some(duration) = self.expire {
            frame.add(Frame::Bulk("PX".into()));
            frame.add(Frame::Bulk(
                Bytes::from(int_as_bytes(&(duration.as_millis() as usize)))
            ));
        }

        frame
    }
}
