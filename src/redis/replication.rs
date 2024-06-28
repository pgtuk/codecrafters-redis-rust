use tokio::sync::oneshot;

use crate::redis::cmd::Command;
use crate::redis::connection::Connection;

pub enum ReplMessage {
    RunCmd(Command),
    AddConnection(Connection)
}

pub(crate) struct ReplManager {
    rx: oneshot::Receiver<ReplMessage>,
    conn_pool: Vec<Connection>
}

impl ReplManager {
    pub(crate) fn new(rx: oneshot::Receiver<ReplMessage>) -> ReplManager {
        ReplManager {
            rx,
            conn_pool: vec![]
        }
    }
    pub(crate) async fn run() {
        // implement command propagation
        todo!()
    }
}
