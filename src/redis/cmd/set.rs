use anyhow::Result;
use bytes::Bytes;

use crate::redis::{
    db::Db,
    frame::Frame,
    parser::Parser,
};


#[derive(Debug, PartialEq)]
pub struct Set {
    key: String,
    value: Bytes,
}

impl Set {
    pub fn new (key: String, value: Bytes) -> Set {
        Set { key, value }
    }

    pub fn parse_args(parser: &mut Parser) -> Result<Set> {
        let key = parser.next_string()?;
        let value = parser.next_bytes()?;

        Ok(Set::new(key, value))
    }

    pub fn apply(self, db: &mut Db) -> Frame {
        db.set(self.key, self.value);

        Set::to_frame()
    }

    fn to_frame() -> Frame {
        Frame::Simple("OK".to_string())
    }
}

