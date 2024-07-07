use std::sync::Arc;

use anyhow::Result;
use tokio::time::{Duration, timeout};

use crate::redis::{frame::Frame, parser::Parser, ServerInfo, utils::Named};
use crate::redis::replica::Replinfo;

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

    pub async fn apply(&self, info: &ServerInfo) -> Frame {
        let mut wait_lock = info.replinfo.wait_lock.lock().await;
        *wait_lock = true;

        let ack = match timeout(
            Duration::from_millis(self.timeout),
            self.wait_for_replication(self.numreplicas, info.replinfo.clone()),
        ).await {
            Ok(_) => self.numreplicas,
            _ => *info.replinfo.repl_completed.read().await,
        };

        let frame = Frame::Integer(ack as u64);

        let mut reset = info.replinfo.repl_completed.write().await;
        *reset = 0;

        frame
    }

    async fn wait_for_replication(&self, n: i8, replinfo: Arc<Replinfo>) -> i8 {
        loop {
            let count = replinfo.repl_completed.read().await;

            if *count >= n {
                return *count;
            }
        }
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
