use tokio::sync::{mpsc, oneshot};

use crate::redis::cmd::Command;

type Receiver = mpsc::Receiver<Message>;

type PropagateSx = oneshot::Sender<Command>;

type HandshakeMsg = oneshot::Sender<Command>;
type WriteMsg = Command;

#[derive(Debug)]
pub(crate) enum Message {
    Handshake(HandshakeMsg),
    Propagate(WriteMsg),
}

pub(crate) struct ChannelManager {
    receiver: Receiver,
    pool: Vec<PropagateSx>
}

impl ChannelManager {
    pub(crate) fn new(receiver: Receiver) -> ChannelManager {
        ChannelManager { receiver, pool: vec![] }
    }

    pub(crate) async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
           match msg {
               Message::Handshake(msg) => self.handle_handshake(msg),
               Message::Propagate(msg) => self.handle_propagate(msg).await,
           };
       }
    }
    async fn handle_propagate(&self, cmd: Command) {
        self.pool.iter().map(|sx| sx.send(cmd));
    }

    fn handle_handshake(&mut self, msg: HandshakeMsg) {
        self.pool.push(msg);
    }
}