use anyhow::Result;

use crate::redis::{
    frame::Frame,
    parser::Parser,
    ServerInfo, utils::Named,
};

use super::ClientCmd;

#[derive(Debug, PartialEq, Clone)]
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

    pub async fn apply(&self, info: &mut ServerInfo) -> Frame {
        info.replinfo.add_replica().await;
        Frame::Simple(format!("FULLRESYNC {} 0", info.replinfo.id))
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
