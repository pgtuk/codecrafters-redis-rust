use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;

use crate::redis::{frame::Frame, parser::Parser, ServerInfo, utils::Named};
use crate::redis::replica::{ReplicationMsg, Replinfo};

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

    pub async fn apply(&self, sender: &mut Arc<Sender<ReplicationMsg>>, server_info: &ServerInfo) -> Frame {
        if !has_pending(&server_info.replinfo).await {
            // if no previous commands were propagated
            // just reply with number of connected replicas
            let repl_count = server_info.replinfo.count.read().await;
            Frame::Integer(*repl_count as u64)
        } else {
            sender.send(ReplicationMsg::Wait(self.timeout)).unwrap();
            sleep(Duration::from_millis(self.timeout)).await;

            let _ = server_info.replinfo.wait_lock.lock().await;
            let ack = *server_info.replinfo.repl_completed.read().await;

            self.reset_repl_counter(server_info).await;

            Frame::Integer(ack as u64)
        }
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

async fn has_pending(replinfo: &Replinfo) -> bool {
    replinfo.has_pending().await
}
