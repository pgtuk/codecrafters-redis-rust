use anyhow::Result;

use crate::redis::{frame::Frame, parser::Parser, ServerInfo, utils::Named};

use super::ClientCmd;

#[derive(Debug, PartialEq, Clone)]
pub struct Wait {
    pub numreplicas: i8,
    pub timeout: u64,
}

impl Named for Wait {
    const NAME: &'static str = "WAIT";
}

impl Wait {
    pub fn parse_args(parser: &mut Parser) -> Result<Wait> {
        let numreplicas = parser.next_string()?.parse::<i8>().unwrap();
        let timeout = parser.next_string()?.parse::<u64>().unwrap();

        Ok(Wait { numreplicas, timeout })
    }

    pub async fn apply(&self, server_info: &ServerInfo) -> Frame {
        let _ = server_info.replinfo.wait_lock.lock().await;

        let ack = *server_info.replinfo.repl_completed.read().await;

        let frame = Frame::Integer(ack as u64);

        self.reset_repl_counter(server_info).await;

        frame
    }

    async fn reset_repl_counter(&self, server_info: &ServerInfo) {
        let mut reset = server_info.replinfo.repl_completed.write().await;
        *reset = 0;
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
