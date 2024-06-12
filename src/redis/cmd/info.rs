use anyhow::Result;

use crate::redis::{
    frame::Frame,
    ServerInfo,
};

#[derive(Debug, PartialEq)]
pub struct Info {}

impl Info {
    pub fn new() -> Info {
        Info { }
    }

    pub fn parse_args() -> Result<Info> {
        Ok(Info::new())
    }

    pub fn apply(self, info: &ServerInfo) -> Frame {
        let string = format!(
            "role:{role}\r\n\
            master_replid:{replid}\r\nmaster_repl_offset:{reploffset}",
            role=info.role,
            replid=info.replinfo.repl_id,
            reploffset=info.replinfo.repl_offset.lock().unwrap(),
        );
        let len = string.len();

        Frame::Bulk(format!("${len}\r\n{string}\r\n").into())
    }
}
