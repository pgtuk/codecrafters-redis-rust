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

    pub fn apply(&self, info: &ServerInfo) -> Frame {
        let string = Info::build_info_string(info);

        Frame::Bulk(string.into())
    }

    fn build_info_string(info: &ServerInfo) -> String {
        format!(
            "role:{role}\nmaster_replid:{replid}\nmaster_repl_offset:{reploffset}",
            role = info.role,
            replid = info.replinfo.id,
            reploffset = info.replinfo.offset.lock().unwrap(),
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
