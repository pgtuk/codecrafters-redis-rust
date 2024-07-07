use std::fmt;

use anyhow::{Error, Result};

use crate::redis::{frame::Frame, parser::Parser, ServerInfo, utils::Named};

use super::ClientCmd;

#[derive(Debug, PartialEq, Clone)]
pub struct Replconf {
    pub param: ReplconfParam,
    pub arg: String,
}

impl Named for Replconf {
    const NAME: &'static str = "REPLCONF";
}

impl Replconf {
    pub fn parse_args(parser: &mut Parser) -> Result<Replconf> {
        let param_as_str = parser.next_string()?.to_lowercase();
        let arg = parser.next_string()?.to_lowercase();
        let param = match &param_as_str[..] {
            "listening-port" => ReplconfParam::ListeningPort,
            "capa" => ReplconfParam::Capa,
            "getack" => ReplconfParam::Getack,
            unknown => return Err(Error::msg(
                format!("Unknown replconf param `{unknown}`")
            )),
        };
        Ok(Replconf { param, arg })
    }

    pub fn apply(&self, info: &ServerInfo) -> Frame {
        match self.param {
            ReplconfParam::ListeningPort | ReplconfParam::Capa => Frame::Simple("OK".to_string()),
            ReplconfParam::Getack => {
                Frame::Array(vec![
                    Frame::Simple("replconf".to_string()),
                    Frame::Simple("ACK".to_string()),
                    Frame::Simple(info.replinfo.offset.blocking_lock().to_string()),
                ])
            }
        }
    }
}

impl ClientCmd for Replconf {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        let items: Vec<String> = vec![
            Replconf::NAME.to_string(),
            self.param.to_string(),
            self.arg.clone(),
        ];

        for item in items {
            frame.add(Frame::Bulk(item.into()))
        }

        frame
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ReplconfParam {
    ListeningPort,
    Capa,
    Getack,
}

impl fmt::Display for ReplconfParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReplconfParam::ListeningPort => write!(f, "listening-port"),
            ReplconfParam::Capa => write!(f, "capa"),
            ReplconfParam::Getack => write!(f, "GETACK")
        }
    }
}