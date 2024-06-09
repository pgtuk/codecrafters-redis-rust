use anyhow::Result;

use crate::redis::{
    db::Db,
    frame::Frame, 
    parser::Parser,
};


#[derive(Debug, PartialEq)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn new(key: String) -> Get {
        Get { key }
    }
 
    pub fn parse_args(parser: &mut Parser) -> Result<Get> {
        match parser.next_string() {
            Ok(key) => Ok(Get::new(key)),
            Err(e) => Err(e.into())
        }
    }

    pub fn apply(self, db: &mut Db) -> Frame {
        match db.get(&self.key) {
            Some(data) => Frame::Bulk(data),
            None => Frame::Null,
        }
    }

    // fn to_frame(data: Option<Bytes>) -> Frame {
    //     unimplemented!()
    // } 
}