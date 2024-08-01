use anyhow::Result;

use crate::redis::{frame::Frame, ServerInfo};
use crate::redis::cmd::ClientCmd;
use crate::redis::utils::Named;

#[derive(Debug, PartialEq, Clone)]
pub struct Info {}

impl Named for Info {
    const NAME: &'static str = "INFO";
}

impl Info {
    pub fn new() -> Info {
        Info {}
    }

    pub fn parse_args() -> Result<Info> {
        Ok(Info::new())
    }

    pub async fn apply(&self, server_info: &ServerInfo) -> Frame {
        let string = Info::build_info_string(server_info).await;
        Frame::Bulk(string.into())
    }

    async fn build_info_string(server_info: &ServerInfo) -> String {
        format!(
            "role:{role}\nmaster_replid:{replid}\nmaster_repl_offset:{reploffset}",
            role = server_info.role,
            replid = server_info.replinfo.id,
            reploffset = server_info.replinfo.offset.lock().await,
        )
    }
}

impl ClientCmd for Info {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        frame.add(Frame::Bulk(Info::NAME.into()));

        frame
    }
}
