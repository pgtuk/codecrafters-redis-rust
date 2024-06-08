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
        self.to_frame(info)
    }
    
    fn to_frame (self, info: &ServerInfo) -> Frame {
        let string = format!(
            "role:{role}",
            role=info.role,
        );
        let len = string.len();

        Frame::Bulk(format!("${len}\r\n{string}\r\n").into())
    }
}
