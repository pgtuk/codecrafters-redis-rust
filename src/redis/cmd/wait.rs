use anyhow::Result;

use crate::redis::{frame::Frame, parser::Parser, utils::Named};

use super::ClientCmd;

#[derive(Debug, PartialEq, Clone)]
pub struct Wait {
    pub numreplicas: i8,
    pub timeout: i32,
}

impl Named for Wait {
    const NAME: &'static str = "WAIT";
}

impl Wait {
    pub fn parse_args(parser: &mut Parser) -> Result<Wait> {
        let numreplicas = parser.next_string()?.parse::<i8>().unwrap();
        let timeout = parser.next_string()?.parse::<i32>().unwrap();

        Ok(Wait { numreplicas, timeout })
    }

    pub async fn apply(&self) -> Frame {
        Frame::Integer(self.numreplicas as u64)
    }
}

impl ClientCmd for Wait {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        let items: Vec<String> = vec![
            Wait::NAME.to_string(),
            self.numreplicas.to_string(),
            self.timeout.to_string(),
        ];

        for item in items {
            frame.add(Frame::Bulk(item.into()))
        }

        frame
    }
}
