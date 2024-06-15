use anyhow::Result;

use super::client_cmd::ClientCmd;
use crate::redis::{
    frame::Frame,
    parser::Parser,
    utils::Named, ServerInfo,
};


#[derive(Debug, PartialEq)]
pub struct Psync {
    pub replication_id: String,
    pub offset: i32,
}

impl Named for Psync {
    const NAME: &'static str = "PSYNC";
}

impl Psync {
    pub fn parse_args(_: &mut Parser) -> Result<Psync> {
        // TODO implement
        Ok(Psync { replication_id: "replication_id".to_string(), offset: 1 })
    }

    pub fn apply(self, info: &ServerInfo) -> Frame {
        Frame::Simple(format!("FULLRESYNC {} 0", info.replinfo.repl_id))
    }
}

impl Default for Psync {
    fn default() -> Self {
        Psync { replication_id: "?".to_string(), offset: -1 }
    }
}

impl ClientCmd for Psync {

    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        let items: Vec<String> = vec![
            Psync::NAME.to_string(), 
            self.replication_id.clone(), 
            self.offset.to_string(),
        ];

        for item in items {
            frame.add(Frame::Bulk(item.into()))
        }
   
        frame
    }
}