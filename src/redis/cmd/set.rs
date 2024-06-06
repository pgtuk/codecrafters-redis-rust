use anyhow::Result;
use bytes::Bytes;
use std::time::Duration;

use crate::redis::{
    db::Db,
    frame::Frame,
    parser::{
        Parser,
        ParserError,
    },
};


#[derive(Debug, PartialEq)]
pub struct Set {
    key: String,
    value: Bytes,

    expire: Option<Duration>,
}

impl Set {
    pub fn new (key: String, value: Bytes, expire: Option<Duration>) -> Set {
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
            },
            Ok(_) => {}
            Err(ParserError::EndOfStream) => {},
            Err(e) => return Err(e.into())
        }

        Ok(Set::new(key, value, expire))
    }

    pub fn apply(self, db: &mut Db) -> Frame {
        db.set(self.key, self.value, self.expire);

        Set::to_frame()
    }

    fn to_frame() -> Frame {
        Frame::Simple("OK".to_string())
    }
}

