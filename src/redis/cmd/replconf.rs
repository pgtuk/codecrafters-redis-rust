use std::fmt;
use anyhow::Result;

use super::client_cmd::ClientCmd;
use crate::redis::{
    frame::Frame,
    parser::Parser,
    utils::Named,
};


#[derive(Debug, PartialEq)]
pub enum ReplconfParam {
    ListeningPort,
    Capa,
}

impl fmt::Display for ReplconfParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReplconfParam::ListeningPort => write!(f, "listening-port"),
            ReplconfParam::Capa => write!(f, "capa"),
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct Replconf {
    pub param: ReplconfParam,
    pub arg: String,
}

impl Named for Replconf {
    const NAME: &'static str = "REPLCONF";
}

impl Replconf {
    pub fn parse_args(_: &mut Parser) -> Result<Replconf> {
        Ok(Replconf { param: ReplconfParam::ListeningPort, arg: "args".to_string() })
    }

    pub fn apply(self) -> Frame {
        Frame::Simple("OK".to_string())
    }
}

impl ClientCmd for Replconf {

    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        let items: Vec<String> = vec![
            Replconf::NAME.to_string(), 
            self.param.to_string(), 
            self.arg.clone()
        ];

        for item in items {
            frame.add(Frame::Bulk(item.into()))
        }
   
        frame
    }
}
